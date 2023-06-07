use std::num::NonZeroU16;
use std::str::FromStr;

mod path;
mod svg;

#[cfg(feature = "serde")]
pub mod wavejson;

pub use path::{AssembledWavePath, PathState, WaveOptions, WavePath, WavePathSegment};
pub use svg::ToSvg;

pub struct Wave {
    name: String,
    cycles: Cycles,
    data: Vec<String>,
    period: NonZeroU16,
    phase: u16,
}

pub struct Figure {
    title: Option<String>,
    footer: Option<String>,

    top_cycle_marker: Option<CycleMarker>,
    bottom_cycle_marker: Option<CycleMarker>,

    hscale: u16,

    lines: Vec<WaveLine>,
}
pub enum WaveLine {
    Group(WaveLineGroup),
    Wave(Wave),
}

pub struct WaveLineGroup(Option<String>, Vec<WaveLine>);
pub struct Cycles(Vec<PathState>);

impl Cycles {
    pub fn new(cycles: Vec<PathState>) -> Self {
        Self(cycles)
    }
}

impl FromIterator<PathState> for Cycles {
    fn from_iter<T: IntoIterator<Item = PathState>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Wave> for WaveLine {
    fn from(wave: Wave) -> Self {
        Self::Wave(wave)
    }
}

impl WaveLine {
    fn render_into<'a>(
        &'a self,
        lines: &'_ mut Vec<AssembledLine<'a>>,
        groups: &'_ mut Vec<WaveGroup<'a>>,
        group_label_at_depth: &mut Vec<bool>,
        has_undefined: &mut bool,
        wave_shape_options: &WaveOptions,
        depth: u32,
    ) -> u32 {
        match self {
            Self::Wave(wave) => {
                if wave.cycles.0.contains(&PathState::X) {
                    *has_undefined = true;
                }

                lines.push(AssembledLine {
                    text: &wave.name,
                    depth,
                    path: WavePath::new(wave.cycles.0.clone(), wave.period, wave.phase, &wave.data)
                        .shape_with_options(wave_shape_options),
                });

                depth
            }
            Self::Group(WaveLineGroup(label, wave_lines)) => {
                // TODO: Do something smarter here.
                if depth > 4 {
                    return depth;
                }

                match group_label_at_depth.get_mut(depth as usize) {
                    None => group_label_at_depth.push(label.is_some()),
                    Some(label_at_level) => *label_at_level |= label.is_some(),
                }

                let mut max_depth = depth + 1;

                let group_start = lines.len();
                for wave_line in wave_lines {
                    let group_depth = wave_line.render_into(
                        lines,
                        groups,
                        group_label_at_depth,
                        has_undefined,
                        wave_shape_options,
                        depth + 1,
                    );

                    if group_depth > max_depth {
                        max_depth = group_depth;
                    }
                }
                let group_end = lines.len();

                groups.push(WaveGroup {
                    depth,
                    label: label.as_ref().map(|s| &s[..]),
                    start: group_start as u32,
                    end: group_end as u32,
                });

                max_depth
            }
        }
    }
}

impl Figure {
    pub fn from_lines<T: Into<WaveLine>>(lines: impl IntoIterator<Item = T>) -> Self {
        Self {
            title: None,
            footer: None,

            top_cycle_marker: None,
            bottom_cycle_marker: None,

            hscale: 1,
            lines: lines.into_iter().map(T::into).collect(),
        }
    }
}

impl FromStr for Cycles {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cycles = Vec::with_capacity(s.len());

        for c in s.chars() {
            let state = match c {
                '1' => PathState::Top,
                '0' => PathState::Bottom,
                'z' => PathState::Middle,
                'x' => PathState::X,
                'p' => PathState::PosedgeClockUnmarked,
                'P' => PathState::PosedgeClockMarked,
                'n' => PathState::NegedgeClockUnmarked,
                'N' => PathState::NegedgeClockMarked,
                '2' => PathState::Box2,
                '3' => PathState::Box3,
                '4' => PathState::Box4,
                '5' => PathState::Box5,
                '6' => PathState::Box6,
                '7' => PathState::Box7,
                '8' => PathState::Box8,
                '9' => PathState::Box9,
                '.' => PathState::Continue,
                '|' => PathState::Gap,
                _ => PathState::X,
            };

            cycles.push(state)
        }

        Ok(Self(cycles))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeMarker {
    None,
    Arrow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockEdge {
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CycleClock {
    edge: ClockEdge,
    has_arrows: bool,
}

pub struct AssembledFigure<'a> {
    num_cycles: u32,

    hscale: u16,

    has_undefined: bool,

    group_label_at_depth: Vec<bool>,
    max_group_depth: u32,

    title: Option<&'a str>,
    footer: Option<&'a str>,

    top_cycle_marker: Option<CycleMarker>,
    bottom_cycle_marker: Option<CycleMarker>,

    pub lines: Vec<AssembledLine<'a>>,
    groups: Vec<WaveGroup<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CycleMarker {
    start: u32,
    every: u32,
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

pub struct WaveGroup<'a> {
    depth: u32,

    label: Option<&'a str>,

    start: u32,
    end: u32,
}

pub struct AssembledLine<'a> {
    text: &'a str,
    depth: u32,
    path: AssembledWavePath,
}

impl AssembledLine<'_> {
    fn is_empty(&self) -> bool {
        self.path.is_empty() && self.text.is_empty()
    }
}

impl WaveGroup<'_> {
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn label(&self) -> Option<&str> {
        self.label
    }
}

impl Figure {
    pub fn new(
        title: Option<String>,
        footer: Option<String>,

        top_cycle_marker: Option<CycleMarker>,
        bottom_cycle_marker: Option<CycleMarker>,

        hscale: u16,
        lines: Vec<WaveLine>,
    ) -> Self {
        Self {
            title,
            footer,

            top_cycle_marker,
            bottom_cycle_marker,

            hscale,
            lines,
        }
    }

    pub fn assemble_with_options(&self, options: &WaveOptions) -> Result<AssembledFigure, ()> {
        let top_cycle_marker = self.top_cycle_marker;
        let bottom_cycle_marker = self.bottom_cycle_marker;
        let hscale = self.hscale;

        let title = self.title.as_ref().map(|s| &s[..]);
        let footer = self.footer.as_ref().map(|s| &s[..]);

        let mut options = options.clone();

        options.cycle_width *= hscale;

        let mut lines = Vec::with_capacity(self.lines.len());
        let mut groups = Vec::new();
        let mut group_label_at_depth = Vec::new();

        let mut has_undefined = false;

        let max_group_depth = self
            .lines
            .iter()
            .map(|line| {
                line.render_into(
                    &mut lines,
                    &mut groups,
                    &mut group_label_at_depth,
                    &mut has_undefined,
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
        let num_cycles = u32::try_from(num_cycles).map_err(|_| ())?;

        Ok(AssembledFigure {
            num_cycles,

            hscale,

            has_undefined,

            group_label_at_depth,
            max_group_depth,

            title,
            footer,

            top_cycle_marker,
            bottom_cycle_marker,

            lines,
            groups,
        })
    }

    #[inline]
    pub fn assemble(&self) -> Result<AssembledFigure, ()> {
        self.assemble_with_options(&WaveOptions::default())
    }
}

impl Wave {
    pub fn new(name: String, cycles: Cycles, data: Vec<String>, period: u16, phase: u16) -> Self {
        let period = NonZeroU16::new(period).unwrap_or(NonZeroU16::new(1).unwrap());

        Self {
            name,
            cycles,
            data,
            period,
            phase,
        }
    }
}

impl<'a> AssembledLine<'a> {
    pub fn depth(&self) -> u32 {
        self.depth
    }
}
