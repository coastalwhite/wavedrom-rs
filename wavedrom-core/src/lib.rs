use std::num::NonZeroU16;
use std::str::FromStr;

#[cfg(feature = "json5")]
pub use json5;

#[cfg(feature = "serde_json")]
pub use serde_json;

mod cycle_offset;
mod path;
pub mod svg;

pub use cycle_offset::{CycleOffset, InCycleOffset};

#[cfg(feature = "serde")]
pub mod wavejson;
pub mod markers;

pub use path::{AssembledSignalPath, CycleState, SignalOptions, SignalPath, SignalPathSegment};

use markers::{GroupMarker, ClockEdge, CycleEnumerationMarker};

#[derive(Debug, Clone)]
pub enum FigureSection {
    Signal(Signal),
    Group(FigureSectionGroup),
}

#[derive(Debug, Clone)]
pub struct FigureSectionGroup(Option<String>, Vec<FigureSection>);

#[derive(Debug, Clone)]
pub struct Signal {
    name: String,
    cycles: Cycles,
    data: Vec<String>,
    period: NonZeroU16,
    phase: CycleOffset,
}

#[derive(Debug, Clone)]
pub struct AssembledLine<'a> {
    text: &'a str,
    depth: u32,
    path: AssembledSignalPath,
}

#[derive(Debug, Clone)]
pub struct Figure {
    title: Option<String>,
    footer: Option<String>,

    top_cycle_marker: Option<CycleEnumerationMarker>,
    bottom_cycle_marker: Option<CycleEnumerationMarker>,

    hscale: u16,

    sections: Vec<FigureSection>,
}

#[derive(Debug, Clone)]
pub struct Cycles(Vec<CycleState>);

impl Cycles {
    pub fn new(cycles: Vec<CycleState>) -> Self {
        Self(cycles)
    }
}

impl FromIterator<CycleState> for Cycles {
    fn from_iter<T: IntoIterator<Item = CycleState>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Signal> for FigureSection {
    fn from(wave: Signal) -> Self {
        Self::Signal(wave)
    }
}

#[derive(Default)]
struct DefinitionTracker {
    has_undefined: bool,
    has_gap: bool,
    has_posedge_marker: bool,
    has_negedge_marker: bool,
}

impl FigureSection {
    fn render_into<'a>(
        &'a self,
        lines: &'_ mut Vec<AssembledLine<'a>>,
        groups: &'_ mut Vec<GroupMarker<'a>>,
        group_label_at_depth: &mut Vec<bool>,
        definitions: &mut DefinitionTracker,
        wave_shape_options: &SignalOptions,
        depth: u32,
    ) -> u32 {
        match self {
            Self::Signal(signal) => {
                for state in signal.cycles() {
                    match state {
                        CycleState::X => definitions.has_undefined = true,
                        CycleState::Gap => definitions.has_gap = true,
                        CycleState::PosedgeClockMarked => definitions.has_posedge_marker = true,
                        CycleState::NegedgeClockMarked => definitions.has_negedge_marker = true,
                        _ => {}
                    }
                }

                lines.push(AssembledLine {
                    text: &signal.name,
                    depth,
                    path: SignalPath::new(signal.cycles(), &signal.data, signal.period, signal.phase)
                        .assemble_with_options(wave_shape_options),
                });

                depth
            }
            Self::Group(FigureSectionGroup(label, sections)) => {
                match group_label_at_depth.get_mut(depth as usize) {
                    None => group_label_at_depth.push(label.is_some()),
                    Some(label_at_level) => *label_at_level |= label.is_some(),
                }

                let mut max_depth = depth + 1;

                let group_start = lines.len();
                for wave_line in sections {
                    let group_depth = wave_line.render_into(
                        lines,
                        groups,
                        group_label_at_depth,
                        definitions,
                        wave_shape_options,
                        depth + 1,
                    );

                    if group_depth > max_depth {
                        max_depth = group_depth;
                    }
                }
                let group_end = lines.len();

                groups.push(GroupMarker::new(
                    group_start as u32,
                    group_end as u32,
                    label.as_ref().map(|s| &s[..]),
                    depth,
                ));

                max_depth
            }
        }
    }
}

impl Figure {
    pub fn from_lines<T: Into<FigureSection>>(lines: impl IntoIterator<Item = T>) -> Self {
        Self {
            title: None,
            footer: None,

            top_cycle_marker: None,
            bottom_cycle_marker: None,

            hscale: 1,
            sections: lines.into_iter().map(T::into).collect(),
        }
    }
}

impl FromStr for Cycles {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cycles = Vec::with_capacity(s.len());

        for c in s.chars() {
            let state = match c {
                '1' => CycleState::Top,
                '0' => CycleState::Bottom,
                'z' => CycleState::Middle,
                'x' => CycleState::X,
                'p' => CycleState::PosedgeClockUnmarked,
                'P' => CycleState::PosedgeClockMarked,
                'n' => CycleState::NegedgeClockUnmarked,
                'N' => CycleState::NegedgeClockMarked,
                '2' => CycleState::Box2,
                '3' => CycleState::Box3,
                '4' => CycleState::Box4,
                '5' => CycleState::Box5,
                '6' => CycleState::Box6,
                '7' => CycleState::Box7,
                '8' => CycleState::Box8,
                '9' => CycleState::Box9,
                '.' => CycleState::Continue,
                '|' => CycleState::Gap,
                '=' => CycleState::Data,
                'u' => CycleState::Up,
                'd' => CycleState::Down,
                _ => CycleState::X,
            };

            cycles.push(state)
        }

        Ok(Self(cycles))
    }
}

pub struct AssembledFigure<'a> {
    num_cycles: u32,

    hscale: u16,

    definitions: DefinitionTracker,

    group_label_at_depth: Vec<bool>,
    max_group_depth: u32,

    title: Option<&'a str>,
    footer: Option<&'a str>,

    top_cycle_marker: Option<CycleEnumerationMarker>,
    bottom_cycle_marker: Option<CycleEnumerationMarker>,

    lines: Vec<AssembledLine<'a>>,
    groups: Vec<GroupMarker<'a>>,
}

impl<'a> AssembledFigure<'a> {
    #[inline]
    fn amount_labels_before(&self, depth: u32) -> u32 {
        self.group_label_at_depth
            .iter()
            .take(depth as usize)
            .filter(|x| **x)
            .count() as u32
    }
}


impl AssembledLine<'_> {
    fn is_empty(&self) -> bool {
        self.path.is_empty() && self.text.is_empty()
    }
}

impl Figure {
    pub fn new(
        title: Option<String>,
        footer: Option<String>,

        top_cycle_marker: Option<CycleEnumerationMarker>,
        bottom_cycle_marker: Option<CycleEnumerationMarker>,

        hscale: u16,
        sections: Vec<FigureSection>,
    ) -> Self {
        Self {
            title,
            footer,

            top_cycle_marker,
            bottom_cycle_marker,

            hscale,
            sections,
        }
    }

    pub fn assemble_with_options(&self, options: &SignalOptions) -> AssembledFigure {
        let top_cycle_marker = self.top_cycle_marker;
        let bottom_cycle_marker = self.bottom_cycle_marker;
        let hscale = self.hscale;

        let title = self.title.as_ref().map(|s| &s[..]);
        let footer = self.footer.as_ref().map(|s| &s[..]);

        let mut options = options.clone();

        options.cycle_width *= hscale;

        let mut lines = Vec::with_capacity(self.sections.len());
        let mut groups = Vec::new();
        let mut group_label_at_depth = Vec::new();

        let mut definitions = DefinitionTracker::default();

        let max_group_depth = self
            .sections
            .iter()
            .map(|line| {
                line.render_into(
                    &mut lines,
                    &mut groups,
                    &mut group_label_at_depth,
                    &mut definitions,
                    &options,
                    0,
                )
            })
            .max()
            .unwrap_or_default();

        let num_cycles = lines
            .iter()
            .map(|line| line.path.num_cycles())
            .max()
            .unwrap_or(0);
        let num_cycles = num_cycles as u32;

        AssembledFigure {
            num_cycles,

            hscale,

            definitions,

            group_label_at_depth,
            max_group_depth,

            title,
            footer,

            top_cycle_marker,
            bottom_cycle_marker,

            lines,
            groups,
        }
    }

    #[inline]
    pub fn assemble(&self) -> AssembledFigure {
        self.assemble_with_options(&SignalOptions::default())
    }
}

impl Signal {
    pub fn new(
        name: String,
        cycles: Cycles,
        data: Vec<String>,
        period: u16,
        phase: CycleOffset,
    ) -> Self {
        let period = NonZeroU16::new(period).unwrap_or(NonZeroU16::MIN);

        Self {
            name,
            cycles,
            data,
            period,
            phase,
        }
    }

    fn cycles(&self) -> &[CycleState] {
        &self.cycles.0
    }
}

impl<'a> AssembledLine<'a> {
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
