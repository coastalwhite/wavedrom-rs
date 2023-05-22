use std::str::FromStr;

mod path;
mod rect;
mod svg;

#[cfg(feature = "wavejson")]
pub mod wavejson;

use path::PathState;

pub use path::{AssembledWavePath, WaveDimension, WavePath, WavePathSegment};
pub use svg::ToSvg;

pub struct Wave {
    pub name: String,
    pub cycles: Cycles,
}

pub struct Figure(Vec<WaveLine>);
pub enum WaveLine {
    Group(Vec<WaveLine>),
    Wave(Wave),
}
pub struct Cycles(pub Vec<CycleData>);

impl From<Wave> for WaveLine {
    fn from(wave: Wave) -> Self {
        Self::Wave(wave)
    }
}

impl WaveLine {
    fn render_into_buffers<'a>(
        &'a self,
        lines_buffer: &'_ mut Vec<AssembledLine<'a>>,
        groups_buffer: &'_ mut Vec<WaveGroup>,
        depth: u32,
    ) -> u32 {
        match self {
            Self::Wave(wave) => {
                lines_buffer.push(AssembledLine {
                    text: &wave.name,
                    group_depth: depth,
                    path: WavePath::new(wave.cycles.0.iter().map(PathState::from).collect()),
                });
                depth
            }
            Self::Group(wave_lines) => {
                // TODO: Do something smarter here.
                if depth > 4 {
                    return depth;
                }

                let mut max_depth = depth + 1;

                let group_start = lines_buffer.len();
                for wave_line in wave_lines {
                    let group_depth = wave_line.render_into_buffers(
                        lines_buffer,
                        groups_buffer,
                        depth + 1,
                    );

                    if group_depth > max_depth {
                        max_depth = group_depth;
                    }
                }
                let group_end = lines_buffer.len();

                groups_buffer.push(WaveGroup {
                    depth,
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
        Self(lines.into_iter().map(T::into).collect())
    }
}

impl FromStr for Cycles {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cycles = Vec::with_capacity(s.len());

        let mut last_state = None;
        for (i, c) in s.char_indices() {
            let state = match c {
                '1' => CycleData::Top,
                '0' => CycleData::Bottom,
                '2' => CycleData::Box(0),
                '3' => CycleData::Box(1),
                '4' => CycleData::Box(2),
                '5' => CycleData::Box(3),
                '.' => last_state.ok_or(i)?,
                _ => return Err(i),
            };

            last_state = Some(state);
            cycles.push(state)
        }

        Ok(Self(cycles))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleData {
    Top,
    Bottom,
    Box(usize),
}

impl From<&CycleData> for PathState {
    fn from(value: &CycleData) -> Self {
        match value {
            CycleData::Top => PathState::Top,
            CycleData::Bottom => PathState::Bottom,
            CycleData::Box(usize) => PathState::Box(*usize),
        }
    }
}

pub struct AssembledFigure<'a> {
    num_cycles: u32,
    max_group_depth: u32,
    lines: Vec<AssembledLine<'a>>,
    groups: Vec<WaveGroup>,
}

struct WaveGroup {
    depth: u32,
    start: u32,
    end: u32,
}

pub struct AssembledLine<'a> {
    text: &'a str,
    group_depth: u32,
    path: WavePath,
}

impl WaveGroup {
    fn len(&self) -> u32 {
        self.end - self.start
    }

    fn is_empty(&self) -> bool {
        self.start == self.end
    }
}


impl Figure {
    pub fn assemble_with_options(&self) -> Result<AssembledFigure, ()> {
        let mut lines = Vec::with_capacity(self.0.len());
        let mut groups = Vec::new();

        let max_group_depth = self
            .0
            .iter()
            .map(|line| line.render_into_buffers(&mut lines, &mut groups, 0))
            .max()
            .unwrap_or_default();

        let num_cycles = lines.iter().map(|line| line.path.len()).max().unwrap_or(0);
        let num_cycles = u32::try_from(num_cycles).map_err(|_| ())?;

        Ok(AssembledFigure {
            num_cycles,

            max_group_depth,

            lines,
            groups,
        })
    }

    #[inline]
    pub fn assemble(&self) -> Result<AssembledFigure, ()> {
        self.assemble_with_options()
    }
}
