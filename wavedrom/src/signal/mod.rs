//! The logic to render signal diagram

mod cycle_offset;
pub mod edges;
mod figure;
pub mod markers;
pub mod options;
mod path;
mod render;

pub use cycle_offset::{CycleOffset, InCycleOffset};
pub use figure::{SignalFigure, SignalFigureSection, SignalFigureSectionGroup};
pub use path::*;
pub use render::*;

use std::num::NonZeroU16;

use edges::LineEdgeMarkers;
use markers::{CycleEnumerationMarker, GroupMarker};

use self::options::PathAssembleOptions;

/// A diagram signal line with a set of cycles.
#[derive(Debug, Clone)]
pub struct Signal {
    name: String,
    cycles: Vec<CycleState>,
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
    path: AssembledSignalPath,
}

#[derive(Default, Debug)]
struct DefinitionTracker {
    has_undefined: bool,
    has_gaps: bool,
    has_posedge_marker: bool,
    has_negedge_marker: bool,
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

impl Default for Signal {
    fn default() -> Self {
        Self {
            name: String::new(),
            cycles: Vec::new(),
            data: Vec::new(),
            node: String::new(),
            period: NonZeroU16::MIN,
            phase: CycleOffset::default(),
        }
    }
}

impl Signal {
    /// Create a new [`Signal`] with a set of parameters.
    pub fn with(
        name: String,
        cycles: Vec<CycleState>,
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

    /// Create a new empty signal
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`Signal`] with the cycles formed from the `states` string.
    #[inline]
    pub fn with_cycle_str(states: impl AsRef<str>) -> Self {
        Signal {
            cycles: states.as_ref().chars().map(CycleState::from).collect(),
            ..Self::default()
        }
    }

    /// Add a set of [`CycleState`]s to the [`Signal`].
    #[inline]
    pub fn add_cycles(mut self, cycles: impl IntoIterator<Item = CycleState>) -> Self {
        self.cycles.extend(cycles);
        self
    }

    /// Add a [`CycleState`] to the [`Signal`].
    #[inline]
    pub fn add_cycle(mut self, cycle: CycleState) -> Self {
        self.cycles.push(cycle);
        self
    }

    /// Add a [`CycleState`] to repeat `n` times to the [`Signal`].
    #[inline]
    pub fn add_n_cycles(mut self, n: usize, cycle: CycleState) -> Self {
        if n == 0 {
            return self;
        }

        self.cycles.reserve(n);

        self.cycles.push(cycle);
        if n > 1 {
            self.cycles.extend(std::iter::repeat(cycle).take(n - 1));
        }

        self
    }

    /// Add a set of nodes to the [`Signal`].
    ///
    /// This is used for putting markers and arrows / edges on the diagram. For more information
    /// look at the [`edges`][crate::edges] documentation.
    #[inline]
    pub fn add_nodes(mut self, nodes: impl AsRef<str>) -> Self {
        self.node.push_str(nodes.as_ref());
        self
    }

    /// Add a node to the [`Signal`].
    ///
    /// This is used for putting markers and arrows / edges on the diagram. For more information
    /// look at the [`edges`][crate::edges] documentation.
    #[inline]
    pub fn add_node(mut self, node: Option<char>) -> Self {
        self.node.push(node.unwrap_or('.'));
        self
    }

    /// Add a set of nodes to the [`Signal`].
    ///
    /// This is used for putting markers and arrows / edges on the diagram. For more information
    /// look at the [`edges`][crate::edges] documentation.
    #[inline]
    pub fn add_data_fields(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.extend(fields.into_iter().map(Into::into));
        self
    }

    /// Add a data field to the [`Signal`].
    ///
    /// This is used to fill up a container state, such as the [`CycleState::Data`] or
    /// [`CycleState::Box2`], with text.
    #[inline]
    pub fn add_data_field(mut self, data: impl Into<String>) -> Self {
        self.data.push(data.into());
        self
    }

    /// Create a [`Signal`] that contains the `state` a number of times. Namely, `repeats` times.
    #[inline]
    pub fn repeated(state: CycleState, repeats: usize) -> Self {
        Signal {
            cycles: vec![state; repeats],
            ..Self::default()
        }
    }

    /// Set the period for a signal. This is mostly important for clock signals.
    #[inline]
    pub fn period(mut self, period: u16) -> Self {
        let period = NonZeroU16::new(period).unwrap_or(NonZeroU16::MIN);

        self.period = period;
        self
    }

    /// Set the phase for a [`Signal`]. This is mostly important for clock signals.
    #[inline]
    pub fn phase(mut self, phase: impl Into<CycleOffset>) -> Self {
        self.phase = phase.into();
        self
    }

    /// Set the name for a [`Signal`]
    #[inline]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get the name of the [`Signal`].
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the nodes of the [`Signal`].
    #[inline]
    pub fn get_nodes(&self) -> &str {
        &self.node
    }

    /// Get the data of the [`Signal`].
    #[inline]
    pub fn get_data_fields(&self) -> &[String] {
        &self.data
    }

    /// Get the phase of the [`Signal`].
    #[inline]
    pub fn get_phase(&self) -> CycleOffset {
        self.phase
    }

    /// Get the period of the [`Signal`].
    #[inline]
    pub fn get_period(&self) -> u16 {
        self.period.get()
    }

    /// Get the period of the [`Signal`].
    #[inline]
    pub(crate) fn get_period_internal(&self) -> NonZeroU16 {
        self.period
    }

    /// Get the cycles that a signal currently contains.
    #[inline]
    pub fn cycles(&self) -> &[CycleState] {
        &self.cycles
    }
}
