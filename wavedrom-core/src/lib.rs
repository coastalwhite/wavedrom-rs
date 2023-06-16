#![deny(rustdoc::broken_intra_doc_links)]
// #![deny(missing_docs)]

use std::num::NonZeroU16;

#[cfg(feature = "json5")]
pub use json5;

#[cfg(feature = "serde_json")]
pub use serde_json;

mod cycle_offset;
mod edges;
mod path;
mod shortcuts;
mod color;
pub mod svg;

pub use color::Color;
pub use cycle_offset::{CycleOffset, InCycleOffset};
pub use shortcuts::*;

pub mod markers;
#[cfg(feature = "serde")]
pub mod wavejson;

pub use edges::{
    EdgeArrowType, EdgeDefinition, EdgeVariant, LineEdgeMarkers, SharpEdgeVariant,
    SplineEdgeVariant,
};
pub use path::{AssembledSignalPath, CycleState, PathAssembleOptions, SignalPath, SignalPathSegment};

use markers::{ClockEdge, CycleEnumerationMarker, GroupMarker};

use self::edges::LineEdgeMarkersBuilder;

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
    node: String,
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

    edges: Vec<EdgeDefinition>,

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

enum SectionItem<'a> {
    GroupStart(u32, &'a FigureSectionGroup),
    GroupEnd(u32),
    Signal(u32, &'a Signal),
}

struct SectionIterator<'a> {
    top_level: std::slice::Iter<'a, FigureSection>,
    sections: Vec<std::slice::Iter<'a, FigureSection>>,
}

impl<'a> SectionIterator<'a> {
    fn new(sections: &'a [FigureSection]) -> Self {
        Self {
            top_level: sections.into_iter(),
            sections: Vec::new(),
        }
    }
}

impl<'a> Iterator for SectionIterator<'a> {
    type Item = SectionItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let depth = self.sections.len() as u32;
        let iter = self.sections.last_mut().unwrap_or(&mut self.top_level);

        Some(match iter.next() {
            None => {
                let _ = self.sections.pop()?;
                SectionItem::GroupEnd(depth)
            }
            Some(FigureSection::Group(group)) => {
                self.sections.push(group.1.iter());
                SectionItem::GroupStart(depth + 1, group)
            }
            Some(FigureSection::Signal(signal)) => SectionItem::Signal(depth, signal),
        })
    }
}

impl Figure {
    pub fn from_lines<T: Into<FigureSection>>(lines: impl IntoIterator<Item = T>) -> Self {
        Self {
            header_text: None,
            footer_text: None,

            top_cycle_marker: None,
            bottom_cycle_marker: None,

            edges: Vec::new(),

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
                'h' => CycleState::HighUnmarked,
                'H' => CycleState::HighMarked,
                'l' => CycleState::LowUnmarked,
                'L' => CycleState::LowMarked,
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

    path_assemble_options: PathAssembleOptions,

    lines: Vec<AssembledLine<'a>>,
    group_markers: Vec<GroupMarker<'a>>,

    line_edge_markers: LineEdgeMarkers<'a>,
}

impl<'a> AssembledFigure<'a> {
    #[inline]
    fn amount_labels_below(&self, depth: u32) -> u32 {
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

        edges: Vec<EdgeDefinition>,
    ) -> Self {
        Self {
            header_text: title,
            footer_text: footer,

            top_cycle_marker,
            bottom_cycle_marker,

            edges,

            hscale,
            sections,
        }
    }

    pub fn assemble_with_options(&self, options: PathAssembleOptions) -> AssembledFigure {
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
        let mut max_group_depth = 0;

        let mut line_edge_markers = LineEdgeMarkersBuilder::new();

        let mut idx = 0;

        let section_iter = SectionIterator::new(&self.sections);
        let mut groups = Vec::new();
        for section_item in section_iter {
            match section_item {
                SectionItem::Signal(depth, signal) => {
                    max_group_depth = u32::max(max_group_depth, depth);

                    line_edge_markers.add_signal(signal);

                    idx += 1;

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
                        .assemble_with_options(options),
                    });
                }
                SectionItem::GroupStart(depth, group) => {
                    match group_label_at_depth.get_mut(depth as usize) {
                        None => group_label_at_depth.push(group.0.is_some()),
                        Some(label_at_level) => *label_at_level |= group.0.is_some(),
                    }

                    groups.push((idx, group));
                },
                SectionItem::GroupEnd(depth) => {
                    let (start_idx, FigureSectionGroup(label, _)) = groups
                        .pop()
                        .expect("A group should be been pushe for this end");

                    group_markers.push(GroupMarker::new(
                        start_idx,
                        idx,
                        label.as_ref().map(|s| &s[..]),
                        depth,
                    ));
                }
            }
        }

        let line_edge_markers = line_edge_markers.build(&self.edges);

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

            path_assemble_options: options,

            lines,
            group_markers,

            line_edge_markers,
        }
    }

    #[inline]
    pub fn assemble(&self) -> AssembledFigure {
        self.assemble_with_options(PathAssembleOptions::default())
    }
}

impl Signal {
    pub fn new(
        name: String,
        cycles: Cycles,
        data: Vec<String>,
        node: String,
        period: u16,
        phase: CycleOffset,
    ) -> Self {
        let period = NonZeroU16::new(period).unwrap_or(NonZeroU16::MIN);

        Self {
            name,
            cycles,
            data,
            node,
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
