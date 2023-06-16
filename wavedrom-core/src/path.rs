use std::num::NonZeroU16;

use crate::color::Color;
use crate::markers::ClockEdgeMarker;
use crate::{ClockEdge, CycleOffset};

#[derive(Debug, Clone)]
pub struct SignalPath<'a> {
    states: Vec<CycleState>,
    period: NonZeroU16,
    phase: CycleOffset,
    data: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleState {
    Top,
    Bottom,
    Middle,
    Box2,
    Box3,
    Box4,
    Box5,
    Box6,
    Box7,
    Box8,
    Box9,
    Data,
    X,
    PosedgeClockUnmarked,
    PosedgeClockMarked,
    NegedgeClockUnmarked,
    NegedgeClockMarked,
    Continue,
    Gap,
    Up,
    Down,
    HighUnmarked,
    HighMarked,
    LowUnmarked,
    LowMarked,
}

#[derive(Debug, Clone)]
pub enum PathCommand {
    LineVertical(i32),
    LineVerticalNoStroke(i32),
    LineHorizontal(i32),
    DashedLineHorizontal(i32),
    Line(i32, i32),
    Curve(i32, i32, i32, i32, i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSegmentBackground {
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct SignalPathSegment {
    x: i32,
    y: i32,
    width: i32,

    is_fully_stroked: bool,
    background: Option<PathSegmentBackground>,

    actions: Vec<PathCommand>,

    text: Option<String>,
    gaps: Vec<CycleOffset>,
    clock_edge_markers: Vec<ClockEdgeMarker>,
}

#[derive(Debug, Clone)]
pub struct PathData {
    current_x: i32,
    current_y: i32,

    start_x: i32,
    start_y: i32,

    is_fully_stroked: bool,
    actions: Vec<PathCommand>,
}

#[derive(Debug, Clone)]
pub struct SignalOptions {
    pub marker_font_size: u32,
    pub marker_color: Color,

    pub name_font_size: u32,
    pub name_color: Color,

    pub path_color: Color,

    pub hint_line_color: Color,

    pub signal_height: u16,
    pub cycle_width: u16,
    pub transition_offset: u16,

    pub undefined_color: Color,
    pub undefined_background: Option<Color>,

    pub backgrounds: [Color; 8],
}

impl Default for SignalOptions {
    fn default() -> Self {
        Self {
            marker_font_size: 14,
            marker_color: Color::BLACK,

            name_font_size: 14,
            name_color: Color::BLACK,

            path_color: Color::BLACK,

            hint_line_color: Color {
                red: 0xCC,
                green: 0xCC,
                blue: 0xCC,
            },

            signal_height: 24,
            cycle_width: 48,
            transition_offset: 4,

            undefined_color: Color::BLACK,
            undefined_background: None,

            backgrounds: [
                Color {
                    red: 0xFF,
                    green: 0xFF,
                    blue: 0xFF,
                },
                Color {
                    red: 0xF7,
                    green: 0xF7,
                    blue: 0xA1,
                },
                Color {
                    red: 0xF9,
                    green: 0xD4,
                    blue: 0x9F,
                },
                Color {
                    red: 0xAD,
                    green: 0xDE,
                    blue: 0xFF,
                },
                Color {
                    red: 0xAC,
                    green: 0xD5,
                    blue: 0xB6,
                },
                Color {
                    red: 0xA4,
                    green: 0xAB,
                    blue: 0xE1,
                },
                Color {
                    red: 0xE8,
                    green: 0xA8,
                    blue: 0xF0,
                },
                Color {
                    red: 0xFB,
                    green: 0xDA,
                    blue: 0xDA,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledSignalPath {
    end_offset: CycleOffset,
    segments: Vec<SignalPathSegment>,
}

impl AssembledSignalPath {
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn num_cycles(&self) -> u32 {
        self.end_offset.cycle_width()
    }
}

impl SignalPathSegment {
    pub fn background(&self) -> Option<&PathSegmentBackground> {
        self.background.as_ref()
    }

    pub fn is_fully_stroked(&self) -> bool {
        self.is_fully_stroked
    }

    pub fn actions(&self) -> &[PathCommand] {
        &self.actions
    }

    pub fn clock_edge_markers(&self) -> &[ClockEdgeMarker] {
        &self.clock_edge_markers
    }

    pub fn gaps(&self) -> &[CycleOffset] {
        &self.gaps
    }

    pub fn marker_text(&self) -> Option<&str> {
        self.text.as_ref().map(|s| &s[..])
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }
}

pub struct SignalSegmentIter<'a> {
    inner: std::slice::Iter<'a, CycleState>,

    cycle_offset: CycleOffset,

    period: NonZeroU16,

    prev: Option<CycleState>,

    forward: PathData,
    backward: PathData,

    box_index: usize,
    box_content: &'a [String],

    clock_edge_markers: Vec<ClockEdgeMarker>,
    gaps: Vec<CycleOffset>,

    options: &'a SignalOptions,
}

#[derive(Debug)]
pub struct SignalSegmentItem {
    pub end_cycle: CycleOffset,
    pub segment: SignalPathSegment,
}

impl<'a> Iterator for SignalSegmentIter<'a> {
    type Item = SignalSegmentItem;

    fn next(&mut self) -> Option<Self::Item> {
        let mut prev = self.prev?;

        loop {
            if let Some(state) = self.inner.next() {
                let state = *state;
                let signal_segment = self.transition(prev, state);

                self.wave_path(state);

                if let Some(signal_segment) = signal_segment {
                    debug_assert_ne!(state, CycleState::Continue);
                    debug_assert_ne!(state, CycleState::Gap);

                    self.prev = Some(state);
                    let segment_item = Some(SignalSegmentItem {
                        end_cycle: self.cycle_offset,
                        segment: signal_segment,
                    });

                    self.cycle_offset += self.cycle_length(state);

                    return segment_item;
                } else {
                    if !matches!(state, CycleState::Continue | CycleState::Gap) {
                        self.prev = Some(state);
                        prev = state;
                    }

                    self.cycle_offset += self.cycle_length(state);
                }
            } else {
                self.prev = None;
                return Some(SignalSegmentItem {
                    end_cycle: self.cycle_offset,
                    segment: self.end(prev),
                });
            }
        }
    }
}

impl<'a> SignalSegmentIter<'a> {
    fn posedge_marker(&mut self) {
        self.clock_edge_markers
            .push(ClockEdgeMarker::new(self.cycle_offset, ClockEdge::Positive));
    }

    fn negedge_marker(&mut self) {
        self.clock_edge_markers
            .push(ClockEdgeMarker::new(self.cycle_offset, ClockEdge::Negative));
    }

    fn gap(&mut self, state: CycleState) {
        self.gaps
            .push(self.cycle_offset + self.cycle_length(state).half())
    }

    fn begin(&mut self, state: CycleState) {
        let t = i32::from(self.options.transition_offset);
        let h = i32::from(self.options.signal_height);

        use CycleState::*;

        match state {
            Top => self.forward.horizontal_line(t),
            Middle => {
                self.forward.restart_move_to(0, h / 2);
                self.forward.horizontal_line(t);
            }
            Bottom => {
                self.forward.restart_move_to(0, h);
                self.forward.horizontal_line(t);
            }
            PosedgeClockMarked | PosedgeClockUnmarked | LowUnmarked | LowMarked => {
                self.forward.restart_move_to(0, h)
            }
            NegedgeClockMarked | NegedgeClockUnmarked | HighUnmarked | HighMarked => {}
            Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X | Continue | Gap => {
                self.forward.horizontal_line(t);
                self.backward.vertical_line_no_stroke(-h);
                self.backward.horizontal_line(-t);
            }
            Up => {
                self.forward.dashed_horizontal_line(t);
            }
            Down => {
                self.forward.restart_move_to(0, h);
                self.forward.dashed_horizontal_line(t);
            }
        }
    }

    fn wave_path(&mut self, mut state: CycleState) {
        let t = i32::from(self.options.transition_offset);
        let h = i32::from(self.options.signal_height);
        let w = i32::from(self.options.cycle_width);
        let p = i32::from(self.period.get());

        use CycleState::*;

        let prev = self.prev.unwrap_or(X);

        if state == Gap {
            self.gap(prev);
        }

        if matches!(state, Continue | Gap) {
            state = prev;
        }

        match state {
            Top | Bottom | Middle => self.forward.horizontal_line(w - t * 2),
            PosedgeClockMarked | PosedgeClockUnmarked => {
                if state == PosedgeClockMarked {
                    self.posedge_marker();
                }

                self.forward.vertical_line(-h);
                self.forward.horizontal_line(w * p / 2);
                self.forward.vertical_line(h);
                self.forward.horizontal_line(w * p / 2);
            }
            NegedgeClockMarked | NegedgeClockUnmarked => {
                if state == NegedgeClockMarked {
                    self.negedge_marker();
                }

                self.forward.vertical_line(h);
                self.forward.horizontal_line(w * p / 2);
                self.forward.vertical_line(-h);
                self.forward.horizontal_line(w * p / 2);
            }
            HighUnmarked | HighMarked | LowUnmarked | LowMarked => {
                self.forward.horizontal_line(w - t);
            }
            Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X => {
                self.forward.horizontal_line(w - t * 2);
                self.backward.horizontal_line(t * 2 - w);
            }
            Continue | Gap => unreachable!(),
            Up | Down => self.forward.dashed_horizontal_line(w - t * 2),
        }
    }

    fn transition(&mut self, state: CycleState, next: CycleState) -> Option<SignalPathSegment> {
        let t = i32::from(self.options.transition_offset);
        let h = i32::from(self.options.signal_height);

        use CycleState::*;

        match (state, next) {
            (Top, Top)
            | (Bottom, Bottom)
            | (Middle, Middle)
            | (Top, Gap | Continue)
            | (Bottom, Gap | Continue)
            | (Middle, Gap | Continue) => self.forward.horizontal_line(t * 2),
            (
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
            ) => {
                self.forward.line(t, h / 2);
                self.backward.line(-t, h / 2);

                let segment = self.commit_with_back_line(state.background());

                self.forward.line(t, -h / 2);
                self.backward.line(-t, -h / 2);

                return Some(segment);
            }
            (Up, Up | Gap | Continue) | (Down, Down | Gap | Continue) => {
                self.forward.dashed_horizontal_line(t * 2)
            }
            (Top, Bottom) => self.forward.line(t * 2, h),
            (Top, Middle) => self.forward.curve(0, h / 2, t, h / 2, t * 2, h / 2),
            (Middle, Top) => self.forward.curve(0, -h / 2, t, -h / 2, t * 2, -h / 2),
            (Middle, Bottom) => self.forward.curve(0, h / 2, t, h / 2, t * 2, h / 2),
            (Bottom, Top) => self.forward.line(t * 2, -h),
            (Bottom, Middle) => self.forward.curve(0, -h / 2, t, -h / 2, t * 2, -h / 2),
            (Bottom, Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data) => {
                self.forward.horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.line(t, -h);
                self.backward.horizontal_line(-t);

                return Some(segment);
            }
            (Middle, Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data) => {
                self.forward.horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.line(t, -h / 2);
                self.backward.line(-t, -h / 2);

                return Some(segment);
            }
            (Top, Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data) => {
                self.forward.horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.horizontal_line(t);
                self.backward.line(-t, -h);

                return Some(segment);
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X, Top) => {
                self.forward.horizontal_line(t);
                self.backward.line(-t, h);

                let segment = self.commit_with_back_line(state.background());

                self.forward.horizontal_line(t);

                return Some(segment);
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X, Middle) => {
                self.forward.curve(0, h / 2, t, h / 2, t * 2, h / 2);
                self.backward.curve(-t * 2 + t, 0, -t * 2, 0, -t * 2, h / 2);

                return Some(self.commit_with_back_line(state.background()));
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X, Bottom) => {
                self.forward.line(t, h);
                self.backward.horizontal_line(-t);

                let segment = self.commit_with_back_line(state.background());

                self.forward.horizontal_line(t);

                return Some(segment);
            }
            (
                PosedgeClockMarked | PosedgeClockUnmarked,
                PosedgeClockMarked | PosedgeClockUnmarked | Gap | Continue,
            ) => {}
            (
                NegedgeClockMarked | NegedgeClockUnmarked,
                NegedgeClockMarked | NegedgeClockUnmarked | Gap | Continue,
            ) => {}
            (
                PosedgeClockMarked | PosedgeClockUnmarked,
                NegedgeClockMarked | NegedgeClockUnmarked,
            ) => self.forward.vertical_line(-h),
            (
                NegedgeClockMarked | NegedgeClockUnmarked,
                PosedgeClockMarked | PosedgeClockUnmarked,
            ) => self.forward.vertical_line(h),
            (
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X,
                PosedgeClockMarked | PosedgeClockUnmarked,
            ) => {
                self.forward.line(t, h);
                self.backward.horizontal_line(-t);

                return Some(self.commit_with_back_line(state.background()));
            }
            (
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X,
                NegedgeClockMarked | NegedgeClockUnmarked,
            ) => {
                self.forward.horizontal_line(t);
                self.backward.line(-t, h);

                return Some(self.commit_with_back_line(state.background()));
            }
            (Bottom, PosedgeClockMarked | PosedgeClockUnmarked) => {
                self.forward.horizontal_line(t);
            }
            (Bottom, NegedgeClockMarked | NegedgeClockUnmarked) => {
                self.forward.line(t, -h);
            }
            (Middle, PosedgeClockMarked | PosedgeClockUnmarked) => {
                self.forward.line(t, h / 2);
            }
            (Middle, NegedgeClockMarked | NegedgeClockUnmarked) => {
                self.forward.line(t, -h / 2);
            }
            (Top, PosedgeClockMarked | PosedgeClockUnmarked) => {
                self.forward.line(t, h);
            }
            (Top, NegedgeClockMarked | NegedgeClockUnmarked) => {
                self.forward.horizontal_line(t);
            }
            (
                PosedgeClockMarked | PosedgeClockUnmarked,
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X,
            ) => {
                let segment = self.commit_without_back_line();

                self.forward.line(t, -h);
                self.backward.horizontal_line(-t);

                return Some(segment);
            }
            (
                NegedgeClockMarked | NegedgeClockUnmarked,
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | Data | X,
            ) => {
                let segment = self.commit_without_back_line();

                self.forward.horizontal_line(t);
                self.backward.line(-t, -h);

                return Some(segment);
            }
            (PosedgeClockMarked | PosedgeClockUnmarked, Bottom) => {
                self.forward.horizontal_line(t);
            }
            (NegedgeClockMarked | NegedgeClockUnmarked, Bottom) => {
                self.forward.line(t, h);
            }
            (PosedgeClockMarked | PosedgeClockUnmarked, Middle) => {
                self.forward.line(t, -h / 2);
            }
            (NegedgeClockMarked | NegedgeClockUnmarked, Middle) => {
                self.forward.line(t, h / 2);
            }
            (PosedgeClockMarked | PosedgeClockUnmarked, Top) => {
                self.forward.line(t, -h);
            }
            (NegedgeClockMarked | NegedgeClockUnmarked, Top) => {
                self.forward.horizontal_line(t);
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data, Gap | Continue) => {
                self.forward.horizontal_line(2 * t);
                self.backward.horizontal_line(-2 * t);
            }
            (Gap | Continue, _) => {
                unreachable!();
            }
            (Up, Top) | (Down, Bottom) => {
                self.forward.dashed_horizontal_line(t);
                self.forward.horizontal_line(t);
            }
            (Top, Up) | (Bottom, Down) => {
                self.forward.horizontal_line(t);
                self.forward.dashed_horizontal_line(t);
            }
            (Up, Bottom | Down) | (Top, Down) => {
                self.forward.curve(t, 0, t, h, t * 2, h);
            }
            (Down, Top | Up) | (Bottom, Up) => {
                self.forward.curve(t, 0, t, -h, t * 2, -h);
            }
            (Up, Middle) => {
                self.forward.curve(t, 0, t, h / 2, t * 2, h / 2);
            }
            (Down, Middle) => {
                self.forward.curve(t, 0, t, -h / 2, t * 2, -h / 2);
            }
            (Middle, Up) => {
                self.forward.curve(t, 0, t, -h / 2, t * 2, -h / 2);
            }
            (Middle, Down) => {
                self.forward.curve(t, 0, t, h / 2, t * 2, h / 2);
            }
            (Up, PosedgeClockUnmarked | PosedgeClockMarked) => {
                self.forward.dashed_horizontal_line(t);
                self.forward.vertical_line_no_stroke(h);
            }
            (PosedgeClockUnmarked | PosedgeClockMarked, Up) => {
                self.forward.vertical_line_no_stroke(-h);
                self.forward.dashed_horizontal_line(t);
            }
            (Down, NegedgeClockUnmarked | NegedgeClockMarked) => {
                self.forward.dashed_horizontal_line(t);
                self.forward.vertical_line_no_stroke(-h);
            }
            (NegedgeClockUnmarked | NegedgeClockMarked, Down) => {
                self.forward.vertical_line_no_stroke(h);
                self.forward.dashed_horizontal_line(t);
            }
            (Up, NegedgeClockUnmarked | NegedgeClockMarked)
            | (NegedgeClockUnmarked | NegedgeClockMarked, Up)
            | (Down, PosedgeClockUnmarked | PosedgeClockMarked)
            | (PosedgeClockUnmarked | PosedgeClockMarked, Down) => {
                self.forward.dashed_horizontal_line(t);
            }
            (Up, Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data) => {
                self.forward.dashed_horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.horizontal_line(t);
                self.backward.line(-t, -h);

                return Some(segment);
            }
            (Down, Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data) => {
                self.forward.dashed_horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.line(t, -h);
                self.backward.horizontal_line(-t);

                return Some(segment);
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data, Up) => {
                self.forward.horizontal_line(t);
                self.backward.line(-t, h);

                let segment = self.commit_with_back_line(state.background());

                self.forward.dashed_horizontal_line(t);

                return Some(segment);
            }
            (Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data, Down) => {
                self.forward.line(t, h);
                self.backward.horizontal_line(-t);

                let segment = self.commit_with_back_line(state.background());

                self.forward.dashed_horizontal_line(t);

                return Some(segment);
            }
            (HighUnmarked | HighMarked | LowUnmarked | LowMarked, Continue | Gap)
            | (Top | HighUnmarked | HighMarked, HighUnmarked | HighMarked)
            | (HighUnmarked | HighMarked, Top)
            | (Bottom | LowUnmarked | LowMarked, LowUnmarked | LowMarked)
            | (LowUnmarked | LowMarked, Bottom) => {
                self.forward.horizontal_line(t);
            }
            (HighUnmarked | HighMarked, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h);
            }
            (LowUnmarked | LowMarked, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(-h);
            }
            (
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
                HighUnmarked | HighMarked,
            ) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.backward.horizontal_line(-t);
                self.backward.vertical_line(h);

                return Some(self.commit_with_back_line(state.background()));
            }
            (
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
                LowUnmarked | LowMarked,
            ) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h);
                self.backward.horizontal_line(-t);

                return Some(self.commit_with_back_line(state.background()));
            }
            (
                HighUnmarked | HighMarked,
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
            ) => {
                self.forward.horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.horizontal_line(t);
                self.backward.line(-t, -h);

                return Some(segment);
            }
            (
                LowUnmarked | LowMarked,
                Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data,
            ) => {
                self.forward.horizontal_line(t);

                let segment = self.commit_without_back_line();

                self.forward.line(t, -h);
                self.backward.horizontal_line(-t);

                return Some(segment);
            }
            (Middle, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(-h / 2);
                self.forward.vertical_line(h);
            }
            (Middle, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h / 2);
                self.forward.vertical_line(-h);
            }
            (Down, LowUnmarked | LowMarked) | (Up, HighUnmarked | HighMarked) => {
                self.forward.dashed_horizontal_line(t);
            }
            (Up, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.dashed_horizontal_line(t);
                self.forward.vertical_line(h);
            }
            (Down, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.dashed_horizontal_line(t);
                self.forward.vertical_line(-h);
            }
            (Bottom, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(-h);
            }
            (Top, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h);
            }
            (PosedgeClockUnmarked | PosedgeClockMarked, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.vertical_line(-h);
            }
            (NegedgeClockUnmarked | NegedgeClockMarked, HighUnmarked | HighMarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.vertical_line(h);
                self.forward.vertical_line(-h);
            }
            (PosedgeClockUnmarked | PosedgeClockMarked, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.vertical_line(-h);
                self.forward.vertical_line(h);
            }
            (NegedgeClockUnmarked | NegedgeClockMarked, LowUnmarked | LowMarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.vertical_line(h);
            }
            (HighUnmarked | HighMarked, PosedgeClockMarked | PosedgeClockUnmarked) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h);
            }
            (LowUnmarked | LowMarked, NegedgeClockMarked | NegedgeClockUnmarked) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(-h);
            }
            (HighUnmarked | HighMarked, NegedgeClockMarked | NegedgeClockUnmarked)
            | (LowUnmarked | LowMarked, PosedgeClockMarked | PosedgeClockUnmarked) => {
                self.forward.horizontal_line(t);
            }
            (HighUnmarked | HighMarked, Up) => {
                self.forward.horizontal_line(t);
                self.forward.dashed_horizontal_line(t);
            }
            (LowUnmarked | LowMarked, Down) => {
                self.forward.horizontal_line(t);
                self.forward.dashed_horizontal_line(t);
            }
            (LowUnmarked | LowMarked, Top) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(-h);
                self.forward.horizontal_line(t);
            }
            (LowUnmarked | LowMarked, Up) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.curve(t, 0, t, -h, 2 * t, -h);
            }
            (HighUnmarked | HighMarked, Bottom) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.horizontal_line(t);
                self.forward.vertical_line(h);
                self.forward.horizontal_line(t);
            }
            (HighUnmarked | HighMarked, Down) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.curve(t, 0, t, h, 2 * t, h);
            }
            (HighUnmarked | HighMarked, Middle) => {
                if matches!(next, HighMarked) {
                    self.posedge_marker();
                }

                self.forward.curve(0, t, t, h / 2, 2 * t, h / 2);
            }
            (LowUnmarked | LowMarked, Middle) => {
                if matches!(next, LowMarked) {
                    self.negedge_marker();
                }

                self.forward.curve(0, -t, t, -h / 2, 2 * t, -h / 2);
            }
        }

        None
    }

    fn end(&mut self, state: CycleState) -> SignalPathSegment {
        let t = i32::from(self.options.transition_offset);
        let h = i32::from(self.options.signal_height);

        use CycleState::*;

        match state {
            Top | Bottom | Middle | HighUnmarked | HighMarked | LowUnmarked | LowMarked => {
                self.forward.horizontal_line(t);
                self.commit_without_back_line()
            }
            PosedgeClockMarked | PosedgeClockUnmarked | NegedgeClockMarked
            | NegedgeClockUnmarked => self.commit_without_back_line(),
            Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9 | X | Data => {
                self.forward.horizontal_line(t);
                self.forward.vertical_line_no_stroke(h);
                self.backward.horizontal_line(-t);
                self.commit_with_back_line(state.background())
            }
            Continue | Gap => unreachable!(),
            Up | Down => {
                self.forward.dashed_horizontal_line(t);
                self.commit_without_back_line()
            }
        }
    }

    fn commit_with_back_line(
        &mut self,
        background: Option<PathSegmentBackground>,
    ) -> SignalPathSegment {
        let segment_start_x = self.forward.start_x;
        let segment_start_y = self.forward.start_y;
        let segment_width = self.forward.current_x - self.forward.start_x;

        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        let is_fully_stroked = self.forward.is_fully_stroked && self.backward.is_fully_stroked;

        // TODO: Optimize this.
        for action in self
            .backward
            .take_and_restart_at(0, 0)
            .actions
            .into_iter()
            .rev()
        {
            self.forward.actions.push(action);
        }

        let text = if background.map_or(false, PathSegmentBackground::is_data_box) {
            let s = self.box_content.get(self.box_index);
            self.box_index += 1;
            s.map(|s| s.clone())
        } else {
            None
        };
        let clock_edge_markers = std::mem::take(&mut self.clock_edge_markers);
        let gaps = std::mem::take(&mut self.gaps);
        let actions = self.forward.take_and_restart_at(start_x, start_y).actions;

        SignalPathSegment {
            x: segment_start_x,
            y: segment_start_y,
            width: segment_width,

            text,
            clock_edge_markers,
            gaps,

            background,
            is_fully_stroked,

            actions,
        }
    }

    fn commit_without_back_line(&mut self) -> SignalPathSegment {
        let segment_start_x = self.forward.start_x;
        let segment_start_y = self.forward.start_y;
        let segment_width = self.forward.current_x - self.forward.start_x;

        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        let clock_edge_markers = std::mem::take(&mut self.clock_edge_markers);
        let gaps = std::mem::take(&mut self.gaps);
        let actions = self.forward.take_and_restart_at(start_x, start_y).actions;

        SignalPathSegment {
            x: segment_start_x,
            y: segment_start_y,
            width: segment_width,

            text: None,
            clock_edge_markers,
            gaps,

            background: None,
            is_fully_stroked: true,
            actions,
        }
    }

    fn cycle_length(&self, mut state: CycleState) -> CycleOffset {
        use CycleState::*;

        if matches!(state, Continue | Gap) {
            state = self.prev.unwrap_or(X);
        }

        CycleOffset::new_rounded(match state {
            Top | Bottom | Middle | Box2 | Box3 | Box4 | Box5 | Box6 | Box7 | Box8 | Box9
            | Data | X | Down | Up | HighUnmarked | HighMarked | LowUnmarked | LowMarked => 1,
            PosedgeClockUnmarked | PosedgeClockMarked | NegedgeClockUnmarked
            | NegedgeClockMarked => self.period.get().into(),
            Continue | Gap => unreachable!(),
        })
    }
}

impl<'a> SignalPath<'a> {
    #[inline]
    pub fn new(
        states: &[CycleState],
        data: &'a [String],
        period: NonZeroU16,
        phase: CycleOffset,
    ) -> Self {
        let states = states.to_vec();

        Self {
            states,
            period,
            data,
            phase,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn assemble_with_options(&self, options: &SignalOptions) -> AssembledSignalPath {
        let mut end_offset = CycleOffset::default();
        let segments = self
            .iter(options)
            .map(|i| {
                end_offset = i.end_cycle;
                i.segment
            })
            .collect();

        AssembledSignalPath {
            end_offset,
            segments,
        }
    }

    #[inline]
    pub fn assemble(&self) -> AssembledSignalPath {
        self.assemble_with_options(&SignalOptions::default())
    }

    pub fn iter(&'a self, options: &'a SignalOptions) -> SignalSegmentIter<'a> {
        let mut iter = SignalSegmentIter {
            inner: self.states.iter(),

            cycle_offset: self.phase,

            period: self.period,

            prev: None,

            forward: PathData::new(
                self.phase.width_offset(u32::from(options.cycle_width)) as i32,
                0,
            ),
            backward: PathData::new(0, 0),

            box_index: 0,
            box_content: self.data,

            clock_edge_markers: Vec::new(),
            gaps: Vec::new(),

            options,
        };

        let Some(first_state) = iter.inner.next() else {
            return iter;
        };

        let first_state = *first_state;

        match first_state {
            CycleState::Continue | CycleState::Gap => iter.prev = Some(CycleState::X),
            _ => iter.prev = Some(first_state),
        }

        iter.begin(first_state);
        iter.wave_path(first_state);

        iter.cycle_offset += iter.cycle_length(first_state);

        iter
    }
}

impl AssembledSignalPath {
    pub fn segments(&self) -> &[SignalPathSegment] {
        &self.segments
    }
}

impl PathCommand {
    pub fn has_no_stroke(&self) -> bool {
        match self {
            Self::LineHorizontal(..)
            | Self::Line(..)
            | Self::Curve(..)
            | Self::LineVertical(..) => false,
            Self::DashedLineHorizontal(..) => false,
            Self::LineVerticalNoStroke(..) => true,
        }
    }
}

impl PathData {
    fn new(x: i32, y: i32) -> Self {
        Self {
            current_x: x,
            current_y: y,

            start_x: x,
            start_y: y,

            is_fully_stroked: true,
            actions: Vec::new(),
        }
    }

    fn horizontal_line(&mut self, dx: i32) {
        self.current_x += dx;

        match self.actions.last_mut() {
            Some(PathCommand::LineHorizontal(ref mut last_dx))
                if dx.signum() == last_dx.signum() =>
            {
                *last_dx += dx
            }
            _ => self.actions.push(PathCommand::LineHorizontal(dx)),
        }
    }

    fn dashed_horizontal_line(&mut self, dx: i32) {
        self.current_x += dx;

        match self.actions.last_mut() {
            Some(PathCommand::DashedLineHorizontal(ref mut last_dx))
                if dx.signum() == last_dx.signum() =>
            {
                *last_dx += dx
            }
            _ => self.actions.push(PathCommand::DashedLineHorizontal(dx)),
        }
    }

    fn line(&mut self, dx: i32, dy: i32) {
        self.current_x += dx;
        self.current_y += dy;

        self.actions.push(PathCommand::Line(dx, dy));
    }

    fn curve(&mut self, cdx1: i32, cdy1: i32, cdx2: i32, cdy2: i32, dx: i32, dy: i32) {
        self.current_x += dx;
        self.current_y += dy;

        self.actions
            .push(PathCommand::Curve(cdx1, cdy1, cdx2, cdy2, dx, dy));
    }

    fn vertical_line_no_stroke(&mut self, dy: i32) {
        self.current_y += dy;
        self.is_fully_stroked = false;
        self.actions.push(PathCommand::LineVerticalNoStroke(dy));
    }

    fn take_and_restart_at(&mut self, x: i32, y: i32) -> PathData {
        let taken = PathData {
            current_x: self.current_x,
            current_y: self.current_y,

            start_x: self.start_x,
            start_y: self.start_y,

            is_fully_stroked: self.is_fully_stroked,
            actions: std::mem::take(&mut self.actions),
        };

        self.current_x = x;
        self.current_y = y;

        self.start_x = x;
        self.start_y = y;

        self.is_fully_stroked = true;

        taken
    }

    fn restart_move_to(&mut self, x: i32, y: i32) {
        self.current_x += x;
        self.current_y += y;

        self.start_x += x;
        self.start_y += y;

        if !self.actions.is_empty() {
            self.actions.clear();
        }
    }

    fn vertical_line(&mut self, dy: i32) {
        self.current_y += dy;

        // There are currently no actions that merge this
        // match self.actions.last_mut() {
        //     Some(PathCommand::LineHorizontal(ref mut last_dx))
        //         if dx.signum() == last_dx.signum() =>
        //     {
        //         *last_dx += dx
        //     }
        //     _ => self.actions.push(PathCommand::LineHorizontal(dx)),
        // }
        self.actions.push(PathCommand::LineVertical(dy));
    }
}

impl CycleState {
    fn background(self) -> Option<PathSegmentBackground> {
        use CycleState::*;

        match self {
            Top | Bottom | Middle | NegedgeClockMarked | NegedgeClockUnmarked
            | PosedgeClockMarked | PosedgeClockUnmarked | Up | Down | HighUnmarked | HighMarked
            | LowUnmarked | LowMarked => None,
            X => Some(PathSegmentBackground::Undefined),
            Box2 | Data => Some(PathSegmentBackground::B2),
            Box3 => Some(PathSegmentBackground::B3),
            Box4 => Some(PathSegmentBackground::B4),
            Box5 => Some(PathSegmentBackground::B5),
            Box6 => Some(PathSegmentBackground::B6),
            Box7 => Some(PathSegmentBackground::B7),
            Box8 => Some(PathSegmentBackground::B8),
            Box9 => Some(PathSegmentBackground::B9),
            Continue | CycleState::Gap => None,
        }
    }
}

impl PathSegmentBackground {
    fn is_data_box(self) -> bool {
        match self {
            Self::Undefined => false,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_cycle_length() {
        macro_rules! assert_cycle_length {
            ([$($item:ident),* $(,)?], $period:literal, ($phase_index:literal, $phase_in_offset:ident) => $result:literal) => {
                let period = NonZeroU16::new($period).unwrap();
                let options = SignalOptions::default();
                let num_cycles = SignalPath::new(
                    &[$(CycleState::$item),*],
                    &[],
                    period,
                    $crate::CycleOffset::new($phase_index, $crate::InCycleOffset::$phase_in_offset),
                ).iter(&options).last().map_or(0, |i| i.end_cycle.cycle_width());
                assert_eq!(num_cycles, $result);
            };
        }

        assert_cycle_length!([], 1, (0, Begin) => 0);
        assert_cycle_length!([], 2, (0, Begin) => 0);
        assert_cycle_length!([Box2], 1, (0, Begin) => 1);
        assert_cycle_length!([Box2], 2, (0, Begin) => 1);
        assert_cycle_length!([PosedgeClockMarked], 1, (0, Begin) => 1);
        assert_cycle_length!([PosedgeClockMarked], 2, (0, Begin) => 2);
        assert_cycle_length!([Box2, PosedgeClockMarked], 3, (0, Begin) => 4);
        assert_cycle_length!([PosedgeClockMarked, NegedgeClockMarked], 3, (0, Begin) => 6);
        assert_cycle_length!([PosedgeClockMarked, Continue, NegedgeClockMarked], 3, (0, Begin) => 9);
    }
}
