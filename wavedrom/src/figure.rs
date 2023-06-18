use crate::{FigureSection, AssembledFigure, PathAssembleOptions, FigureSectionGroup, Signal, CycleState, DefinitionTracker, AssembledLine, SignalPath};
use crate::edges::{EdgeDefinition, EdgeVariant, LineEdgeMarkersBuilder};
use crate::markers::{CycleEnumerationMarker, GroupMarker};

impl Default for Figure {
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
/// use wavedrom::{Figure, Signal, CycleState};
///
/// let figure = Figure::new()
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
#[doc=include_str!("../assets/doc-figure-example.svg")]
///
/// [dtd]: https://en.wikipedia.org/wiki/Digital_timing_diagram
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

impl Figure {
    /// Create a new [`Figure`] with a set of parameters.
    pub fn with(
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
        self.sections.push(FigureSection::Signal(signal));
        self
    }

    /// Add a set of [`Signal`] lines to the [`Figure`].
    pub fn add_signals(mut self, signals: impl IntoIterator<Item = Signal>) -> Self {
        self.sections
            .extend(signals.into_iter().map(FigureSection::Signal));
        self
    }

    /// Add a [`FigureSection`] to the [`Figure`].
    pub fn add_section(mut self, section: FigureSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Add a set of [`FigureSection`]s to the [`Figure`].
    pub fn add_sections(mut self, sections: impl IntoIterator<Item = FigureSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Add a [`FigureSectionGroup`] to the [`Figure`].
    pub fn add_group(mut self, group: FigureSectionGroup) -> Self {
        self.sections.push(FigureSection::Group(group));
        self
    }

    /// Add a set of [`FigureSectionGroup`]s to the [`Figure`].
    pub fn add_groups(mut self, groups: impl IntoIterator<Item = FigureSectionGroup>) -> Self {
        self.sections
            .extend(groups.into_iter().map(FigureSection::Group));
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
                            CycleState::PosedgeClockMarked => definitions.has_posedge_marker = true,
                            CycleState::NegedgeClockMarked => definitions.has_negedge_marker = true,
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
            Some(FigureSection::Group(group)) => {
                self.sections.push(group.1.iter());
                SectionItem::GroupStart(depth + 1, group)
            }
            Some(FigureSection::Signal(signal)) => SectionItem::Signal(depth, signal),
        })
    }
}
