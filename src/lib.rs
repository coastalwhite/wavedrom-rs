use std::str::FromStr;

mod path;
mod rect;
mod svg;

#[cfg(feature = "wavejson")]
pub mod wavejson;

use path::PathState;

pub use path::{AssembledWavePath, WaveDimension, WavePath, WavePathSegment};
pub use svg::ToSvg;

use self::path::BoxData;

pub struct Wave {
    pub name: String,
    pub cycles: Cycles,
    pub data: Vec<String>,
}

pub struct Figure(Vec<WaveLine>);
pub enum WaveLine {
    Group(WaveLineGroup),
    Wave(Wave),
}

pub struct WaveLineGroup(Option<String>, Vec<WaveLine>);
pub struct Cycles(pub Vec<CycleData>);

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
        depth: u32,
    ) -> u32 {
        match self {
            Self::Wave(wave) => {
                if wave.cycles.0.contains(&CycleData::Undefined) {
                    *has_undefined = true;
                }

                lines.push(AssembledLine {
                    text: &wave.name,
                    depth,
                    path: WavePath::new(wave.cycles.0.iter().map(PathState::from).collect()),
                    data: get_data_locations(wave),
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
                'z' | 'Z' => CycleData::Middle,
                'x' | 'X' => CycleData::Undefined,
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
    Middle,
    Undefined,
    
    Box(usize),
}

impl From<&CycleData> for PathState {
    fn from(value: &CycleData) -> Self {
        match value {
            CycleData::Top => PathState::Top,
            CycleData::Bottom => PathState::Bottom,
            CycleData::Middle => PathState::Middle,
            CycleData::Undefined => PathState::Box(BoxData::Undefined),
            CycleData::Box(usize) => PathState::Box(BoxData::Index(*usize)),
        }
    }
}

pub struct AssembledFigure<'a> {
    num_cycles: u32,

    has_undefined: bool,

    group_label_at_depth: Vec<bool>,
    max_group_depth: u32,

    lines: Vec<AssembledLine<'a>>,
    groups: Vec<WaveGroup<'a>>,
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

struct WaveGroup<'a> {
    depth: u32,

    label: Option<&'a str>,

    start: u32,
    end: u32,
}

fn get_data_locations(wave: &Wave) -> Vec<(u32, u32, Option<&str>)> {
    // Temporary

    struct Current {
        start: u32,
        idx: usize,
    }

    let mut items = Vec::new();
    let mut current = None;
    for (offset, cycle) in (0..u32::MAX).zip(wave.cycles.0.iter()) {
        use CycleData::*;

        match (&current, cycle) {
            (
                Some(Current {
                    idx: current_idx, ..
                }),
                Box(idx),
            ) if current_idx == idx => {}
            (
                Some(Current {
                    idx: current_idx,
                    start,
                }),
                Box(idx),
            ) => {
                items.push((
                    *start,
                    offset,
                    wave.data.get(*current_idx).as_ref().map(|s| &s[..]),
                ));
                current = Some(Current {
                    start: offset,
                    idx: *idx,
                });
            }
            (
                Some(Current {
                    idx: current_idx,
                    start,
                }),
                _,
            ) => {
                items.push((
                    *start,
                    offset,
                    wave.data.get(*current_idx).as_ref().map(|s| &s[..]),
                ));
                current = None;
            }

            (None, Box(idx)) => {
                current = Some(Current {
                    start: offset,
                    idx: *idx,
                })
            }
            (None, _) => {}
        }
    }

    if let Some(Current { idx, start }) = current {
        items.push((
            start,
            wave.cycles.0.len() as u32,
            wave.data.get(idx).as_ref().map(|s| &s[..]),
        ));
    }

    items
}

pub struct AssembledLine<'a> {
    text: &'a str,
    depth: u32,
    path: WavePath,
    data: Vec<(u32, u32, Option<&'a str>)>,
}

impl AssembledLine<'_> {
    fn is_empty(&self) -> bool {
        self.path.is_empty() && self.text.is_empty()
    }
}

impl WaveGroup<'_> {
    fn len(&self) -> u32 {
        self.end - self.start
    }

    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    fn label(&self) -> Option<&str> {
        self.label
    }
}

impl Figure {
    pub fn assemble_with_options(&self) -> Result<AssembledFigure, ()> {
        let mut lines = Vec::with_capacity(self.0.len());
        let mut groups = Vec::new();
        let mut group_label_at_depth = Vec::new();

        let mut has_undefined = false;

        let max_group_depth = self
            .0
            .iter()
            .map(|line| {
                line.render_into(
                    &mut lines,
                    &mut groups,
                    &mut group_label_at_depth,
                    &mut has_undefined,
                    0,
                )
            })
            .max()
            .unwrap_or_default();

        let num_cycles = lines.iter().map(|line| line.path.len()).max().unwrap_or(0);
        let num_cycles = u32::try_from(num_cycles).map_err(|_| ())?;

        Ok(AssembledFigure {
            num_cycles,

            has_undefined,

            group_label_at_depth,
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
