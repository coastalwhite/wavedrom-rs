use super::edges::{EdgeDefinition, EdgeVariant, LineEdgeMarkersBuilder};
use super::markers::{CycleEnumerationMarker, GroupMarker};
use super::options::PathAssembleOptions;
use super::{AssembledFigure, AssembledLine, CycleState, DefinitionTracker, Signal, SignalPath};

impl Default for SignalFigure {
    fn default() -> Self {
        Self {
            header_text: None,
            footer_text: None,
            top_cycle_marker: None,
            bottom_cycle_marker: None,
            hscale: 1,
            edges: Vec::new(),
            sections: Vec::new(),
        }
    }
}

/// An encapsulation of everything to form a [Digital Timing Diagram][dtd]
///
/// There are two ways to utilize [`Figure`].
///
/// 1. Utilzing [`Figure::new`] and the builder pattern.
/// 2. Utilizing [`Figure::with`] and defining all parameters.
///
/// # Examples
///
/// ```
/// use std::fs::File;
/// use wavedrom::Figure;
/// use wavedrom::signal::{Signal, SignalFigure, CycleState};
///
/// let figure = SignalFigure::new()
///                  .header_text("Hello World!")
///                  .footer_text("Bye World!")
///                  .add_signals([
///                      Signal::repeated(CycleState::X,    8),
///                      Signal::repeated(CycleState::Box2, 7),
///                      Signal::repeated(CycleState::Box3, 9),
///                      Signal::repeated(CycleState::Box4, 4),
///                  ]);
///
/// let assembled_figure = figure.assemble();
/// # #[allow(unused)]
/// let path = "path/to/file.svg";
/// # let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/doc-figure-example.svg");
/// let mut file = File::create(path)?;
/// assembled_figure.write_svg(&mut file)?;
/// # <Result<(), std::io::Error>>::Ok(())
/// ```
///
/// **Result:**
///
#[doc=include_str!("../../assets/doc-figure-example.svg")]
///
/// [dtd]: https://en.wikipedia.org/wiki/Digital_timing_diagram
#[derive(Debug, Clone)]
pub struct SignalFigure {
    header_text: Option<String>,
    footer_text: Option<String>,

    top_cycle_marker: Option<CycleEnumerationMarker>,
    bottom_cycle_marker: Option<CycleEnumerationMarker>,

    hscale: u16,

    edges: Vec<EdgeDefinition>,

    sections: Vec<SignalFigureSection>,
}

/// A section of the figure's signals
#[derive(Debug, Clone)]
pub enum SignalFigureSection {
    /// A [`Signal`]
    Signal(Signal),
    /// A group of [`Signal`]s
    Group(SignalFigureSectionGroup),
}

/// A section of the figure's group
#[derive(Debug, Clone)]
pub struct SignalFigureSectionGroup(Option<String>, Vec<SignalFigureSection>);

impl SignalFigureSectionGroup {
    /// Create a new [`SignalFigureSectionGroup`]
    pub fn new(label: Option<String>, items: Vec<SignalFigureSection>) -> SignalFigureSectionGroup {
        Self(label, items)
    }
}

impl SignalFigure {
    /// Create a new [`Figure`] with a set of parameters.
    pub fn with(
        title: Option<String>,
        footer: Option<String>,

        top_cycle_marker: Option<CycleEnumerationMarker>,
        bottom_cycle_marker: Option<CycleEnumerationMarker>,

        hscale: u16,
        sections: Vec<SignalFigureSection>,

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

    /// Create a new empty [`Figure`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Give a header text to the [`Figure`].
    #[inline]
    pub fn header_text(mut self, header_text: impl Into<String>) -> Self {
        self.header_text = Some(header_text.into());
        self
    }

    /// Give a footer text to the [`Figure`].
    #[inline]
    pub fn footer_text(mut self, footer_text: impl Into<String>) -> Self {
        self.footer_text = Some(footer_text.into());
        self
    }

    /// Give a top cycle enumeration marker to the figure with starting with clock cycle `start`
    /// and displaying one marker every `every`th cycle.
    #[inline]
    pub fn top_cycle_marker(mut self, start: u32, every: u32) -> Self {
        self.top_cycle_marker = Some(CycleEnumerationMarker::new(start, every));
        self
    }

    /// Give a bottom cycle enumeration marker to the figure with starting with clock cycle `start`
    /// and displaying one marker every `every`th cycle.
    #[inline]
    pub fn bottom_cycle_marker(mut self, start: u32, every: u32) -> Self {
        self.bottom_cycle_marker = Some(CycleEnumerationMarker::new(start, every));
        self
    }

    /// Set the horizontal scale of the [`Figure`].
    #[inline]
    pub fn horizontal_scale(mut self, hscale: u16) -> Self {
        self.hscale = hscale;
        self
    }

    /// Add a labeled arrow / edge with a set of parameters to the [`Figure`].
    pub fn add_labeled_edge_with(
        mut self,
        variant: EdgeVariant,
        from: char,
        to: char,
        label: impl Into<String>,
    ) -> Self {
        self.edges
            .push(EdgeDefinition::new(variant, from, to, Some(label.into())));
        self
    }

    /// Add a unlabeled arrow / edge with a set of parameters to the [`Figure`].
    pub fn add_edge_with(mut self, variant: EdgeVariant, from: char, to: char) -> Self {
        self.edges
            .push(EdgeDefinition::new(variant, from, to, None));
        self
    }

    /// Add an arrow / edge to the [`Figure`].
    pub fn add_edge(mut self, edge: EdgeDefinition) -> Self {
        self.edges.push(edge);
        self
    }

    /// Add a set of arrows / edges to the [`Figure`].
    pub fn add_edges(mut self, edges: impl IntoIterator<Item = EdgeDefinition>) -> Self {
        self.edges.extend(edges);
        self
    }

    /// Add a [`Signal`] line to the [`Figure`].
    pub fn add_signal(mut self, signal: Signal) -> Self {
        self.sections.push(SignalFigureSection::Signal(signal));
        self
    }

    /// Add a set of [`Signal`] lines to the [`Figure`].
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = Signal>) -> Self {
        self.sections
            .extend(signals.into_iter().map(SignalFigureSection::Signal));
        self
    }

    /// Add a [`SignalFigureSection`] to the [`Figure`].
    pub fn add_section(mut self, section: SignalFigureSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a set of [`SignalFigureSection`]s to the [`Figure`].
    pub fn add_sections(mut self, sections: impl IntoIterator<Item = SignalFigureSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add a [`SignalFigureSectionGroup`] to the [`Figure`].
    pub fn add_group(mut self, group: SignalFigureSectionGroup) -> Self {
        self.sections.push(SignalFigureSection::Group(group));
        self
    }

    /// Add a set of [`SignalFigureSectionGroup`]s to the [`Figure`].
    pub fn add_groups(
        mut self,
        groups: impl IntoIterator<Item = SignalFigureSectionGroup>,
    ) -> Self {
        self.sections
            .extend(groups.into_iter().map(SignalFigureSection::Group));
        self
    }

    /// Form the signal paths and fetch information needed for rendering with a certain set of
    /// [`PathAssembleOptions`].
    ///
    /// Calling this with the default [`PathAssembleOptions`] is equivalent to calling
    /// [`Figure::assemble`]. [`PathAssembleOptions`].
    ///
    /// The assembly step is a necessary step to render the [`Digital Timing Diagram`][dtd]. A more
    /// detailed description can be found in the [root level documentation][crate].
    ///
    /// [dtd]: https://en.wikipedia.org/wiki/Digital_timing_diagram
    pub fn assemble_with_options(&self, mut options: PathAssembleOptions) -> AssembledFigure {
        let top_cycle_marker = self.top_cycle_marker;
        let bottom_cycle_marker = self.bottom_cycle_marker;
        let hscale = self.hscale;

        let header_text = self.header_text.as_ref().map(|s| &s[..]);
        let footer_text = self.footer_text.as_ref().map(|s| &s[..]);

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
                            // NOTE: This is overeager for the definition on the High and Low
                            // marked states, but this is fine for now.
                            CycleState::PosedgeClockMarked | CycleState::HighMarked => {
                                definitions.has_posedge_marker = true
                            }
                            CycleState::NegedgeClockMarked | CycleState::LowMarked => {
                                definitions.has_negedge_marker = true
                            }
                            _ => {}
                        }
                    }

                    lines.push(AssembledLine {
                        text: signal.get_name(),
                        path: SignalPath::new(
                            signal.cycles(),
                            signal.get_data_fields(),
                            signal.get_period_internal(),
                            signal.get_phase(),
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
                }
                SectionItem::GroupEnd(depth) => {
                    let (start_idx, SignalFigureSectionGroup(label, _)) = groups
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

    /// Form the signal paths and fetch information needed for rendering.
    ///
    /// This is equivalent to calling [`Figure::assemble_with_options`] with the default
    /// [`PathAssembleOptions`].
    ///
    /// The assembly step is a necessary step to render the [`Digital Timing Diagram`][dtd]. A more
    /// detailed description can be found in the [root level documentation][crate].
    ///
    /// [dtd]: https://en.wikipedia.org/wiki/Digital_timing_diagram
    #[inline]
    pub fn assemble(&self) -> AssembledFigure {
        self.assemble_with_options(PathAssembleOptions::default())
    }
}

impl From<Signal> for SignalFigureSection {
    fn from(wave: Signal) -> Self {
        Self::Signal(wave)
    }
}

enum SectionItem<'a> {
    GroupStart(u32, &'a SignalFigureSectionGroup),
    GroupEnd(u32),
    Signal(u32, &'a Signal),
}

struct SectionIterator<'a> {
    top_level: std::slice::Iter<'a, SignalFigureSection>,
    sections: Vec<std::slice::Iter<'a, SignalFigureSection>>,
}

impl<'a> SectionIterator<'a> {
    fn new(sections: &'a [SignalFigureSection]) -> Self {
        Self {
            top_level: sections.iter(),
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
            Some(SignalFigureSection::Group(group)) => {
                self.sections.push(group.1.iter());
                SectionItem::GroupStart(depth + 1, group)
            }
            Some(SignalFigureSection::Signal(signal)) => SectionItem::Signal(depth, signal),
        })
    }
}
