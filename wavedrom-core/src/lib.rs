#![deny(rustdoc::broken_intra_doc_links)]
// #![deny(missing_docs)]

use std::num::NonZeroU16;

#[cfg(feature = "json5")]
pub use json5;

#[cfg(feature = "serde_json")]
pub use serde_json;

mod cycle_offset;
mod path;
mod shortcuts;
pub mod svg;

pub use cycle_offset::{CycleOffset, InCycleOffset};
pub use shortcuts::*;

pub mod markers;
#[cfg(feature = "serde")]
pub mod wavejson;

pub use path::{AssembledSignalPath, CycleState, SignalOptions, SignalPath, SignalPathSegment};

use markers::{ClockEdge, CycleEnumerationMarker, GroupMarker};

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

/// A line of the [`AssembledFigure`].
///
/// This contains the shaped signal path, the group nesting depth and the name of the signal line.
#[derive(Debug, Clone)]
pub struct AssembledLine<'a> {
    text: &'a str,
    depth: u32,
    path: AssembledSignalPath,
}

#[derive(Debug, Clone)]
pub struct Figure {
    header_text: Option<String>,
    footer_text: Option<String>,

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

#[derive(Default, Debug)]
struct DefinitionTracker {
    has_undefined: bool,
    has_gaps: bool,
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
                // If the first state is a Gap or Continue this is also an undefined state.
                if signal.cycles().first().map_or(false, |state| {
                    matches!(state, CycleState::Continue | CycleState::Gap)
                }) {
                    definitions.has_undefined = true;
                }

                for state in signal.cycles() {
                    match state {
                        CycleState::X => definitions.has_undefined = true,
                        CycleState::Gap => definitions.has_gaps = true,
                        CycleState::PosedgeClockMarked => definitions.has_posedge_marker = true,
                        CycleState::NegedgeClockMarked => definitions.has_negedge_marker = true,
                        _ => {}
                    }
                }

                lines.push(AssembledLine {
                    text: &signal.name,
                    depth,
                    path: SignalPath::new(
                        signal.cycles(),
                        &signal.data,
                        signal.period,
                        signal.phase,
                    )
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
            header_text: None,
            footer_text: None,

            top_cycle_marker: None,
            bottom_cycle_marker: None,

            hscale: 1,
            sections: lines.into_iter().map(T::into).collect(),
        }
    }
}

impl Cycles {
    fn from_str(s: &str) -> Self {
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

        Self(cycles)
    }
}

/// A [`Figure`] that has been assembled with the [`Figure::assemble`] or
/// [`Figure::assemble_with_options`] methods.
///
/// An assembled figure contains all the information necessary to perform rendering.
#[derive(Debug)]
pub struct AssembledFigure<'a> {
    num_cycles: u32,

    hscale: u16,

    definitions: DefinitionTracker,

    group_label_at_depth: Vec<bool>,
    max_group_depth: u32,

    header_text: Option<&'a str>,
    footer_text: Option<&'a str>,

    top_cycle_marker: Option<CycleEnumerationMarker>,
    bottom_cycle_marker: Option<CycleEnumerationMarker>,

    lines: Vec<AssembledLine<'a>>,
    group_markers: Vec<GroupMarker<'a>>,
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

    /// Returns the maximum cycle width over all lines.
    #[inline]
    pub fn num_cycles(&self) -> u32 {
        self.num_cycles
    }

    /// Returns the scaling factor for the horizontal axis.
    #[inline]
    pub fn horizontal_scale(&self) -> u16 {
        self.hscale
    }

    /// Returns whether the [`AssembledFigure`] contains any [`CycleState::X`]
    #[inline]
    pub fn has_undefined(&self) -> bool {
        self.definitions.has_undefined
    }

    /// Returns whether the [`AssembledFigure`] contains any [`CycleState::Gap`]
    #[inline]
    pub fn has_gaps(&self) -> bool {
        self.definitions.has_gaps
    }

    /// Returns whether the [`AssembledFigure`] contains any [`CycleState::PosedgeClockMarked`]
    #[inline]
    pub fn has_posedge_marker(&self) -> bool {
        self.definitions.has_posedge_marker
    }

    /// Returns whether the [`AssembledFigure`] contains any [`CycleState::NegedgeClockMarked`]
    #[inline]
    pub fn has_negedge_marker(&self) -> bool {
        self.definitions.has_negedge_marker
    }

    /// Returns the whether there is a label at group nesting level `depth`.
    #[inline]
    pub fn has_group_label_at_depth(&self, depth: u32) -> bool {
        let Ok(depth) = usize::try_from(depth) else {
            return false;
        };

        self.group_label_at_depth
            .get(depth)
            .cloned()
            .unwrap_or(false)
    }

    /// Returns the maximum depth of the group nesting.
    #[inline]
    pub fn group_nesting(&self) -> u32 {
        self.max_group_depth
    }

    /// Returns the lines that the [`AssembledFigure`] contains
    #[inline]
    pub fn lines(&self) -> &[AssembledLine<'a>] {
        &self.lines
    }

    /// Returns the markers for the group nestings
    #[inline]
    pub fn group_markers(&self) -> &[GroupMarker<'a>] {
        &self.group_markers
    }

    /// Returns a potential header text of the [`AssembledFigure`]
    #[inline]
    pub fn header_text(&self) -> Option<&'a str> {
        self.header_text
    }

    /// Returns a potential footer text of the [`AssembledFigure`]
    #[inline]
    pub fn footer_text(&self) -> Option<&'a str> {
        self.footer_text
    }

    /// Returns a [`CycleEnumerationMarker`] above the signals of the [`AssembledFigure`]
    #[inline]
    pub fn top_cycle_marker(&self) -> Option<CycleEnumerationMarker> {
        self.top_cycle_marker
    }

    /// Returns a [`CycleEnumerationMarker`] below the signals of the [`AssembledFigure`]
    #[inline]
    pub fn bottom_cycle_marker(&self) -> Option<CycleEnumerationMarker> {
        self.bottom_cycle_marker
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
            header_text: title,
            footer_text: footer,

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

        let header_text = self.header_text.as_ref().map(|s| &s[..]);
        let footer_text = self.footer_text.as_ref().map(|s| &s[..]);

        let mut options = options.clone();

        options.cycle_width *= hscale;

        let mut lines = Vec::with_capacity(self.sections.len());
        let mut group_markers = Vec::new();
        let mut group_label_at_depth = Vec::new();

        let mut definitions = DefinitionTracker::default();

        let max_group_depth = self
            .sections
            .iter()
            .map(|line| {
                line.render_into(
                    &mut lines,
                    &mut group_markers,
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

            header_text,
            footer_text,

            top_cycle_marker,
            bottom_cycle_marker,

            lines,
            group_markers,
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
