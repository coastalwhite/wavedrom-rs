pub struct WavePath(Vec<PathState>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathState {
    Top,
    Bottom,
    Box(usize),
}

#[derive(Debug, Clone)]
pub enum PathCommand {
    LineVerticalNoStroke(i32),
    LineHorizontal(i32),
    Line(i32, i32),
}

#[derive(Debug, Clone)]
struct PathSegmentEncasement {
    data_index: usize,
    is_fully_stroked: bool,
}

#[derive(Debug, Clone)]
enum PathSegmentCloseStatus {
    Encased(PathSegmentEncasement),
    Open,
}

#[derive(Debug, Clone)]
pub struct WavePathSegment {
    x: i32,
    y: i32,

    close_status: PathSegmentCloseStatus,
    actions: Vec<PathCommand>,
}

#[derive(Debug, Clone)]
pub struct PathData {
    current_x: i32,
    current_y: i32,

    start_x: i32,
    start_y: i32,

    is_fully_stroked: bool,
    pub(crate) actions: Vec<PathCommand>,
}

#[derive(Debug)]
pub struct PathString {
    forward: PathData,
    backward: PathData,
    segments: Vec<WavePathSegment>,
}

#[derive(Debug, Clone)]
pub struct WaveDimension {
    pub wave_height: u16,
    pub cycle_width: u16,
    pub transition_offset: u16,
}

impl Default for WaveDimension {
    fn default() -> Self {
        Self {
            wave_height: 16,
            cycle_width: 20,
            transition_offset: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledWavePath {
    segments: Vec<WavePathSegment>,
}

impl WavePathSegment {
    pub fn is_open(&self) -> bool {
        matches!(self.close_status, PathSegmentCloseStatus::Open)
    }

    pub fn data_index(&self) -> Option<usize> {
        match self.close_status {
            PathSegmentCloseStatus::Encased(PathSegmentEncasement { data_index, .. }) => {
                Some(data_index)
            }
            _ => None,
        }
    }

    pub fn is_fully_stroked(&self) -> bool {
        match self.close_status {
            PathSegmentCloseStatus::Encased(PathSegmentEncasement {
                is_fully_stroked, ..
            }) => is_fully_stroked,
            _ => true,
        }
    }

    pub fn actions(&self) -> &[PathCommand] {
        &self.actions
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }
}

impl WavePath {
    #[inline]
    pub fn new(states: Vec<PathState>) -> Self {
        Self(states)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn render_with_options(&self, options: &WaveDimension) -> AssembledWavePath {
        let mut state_iter = self.0.iter();
        let Some(mut last_state) = state_iter.next() else {
            return AssembledWavePath { segments: Vec::new() };
        };

        let mut current_path_string = PathString::new(0, 0);

        last_state.begin(&options, &mut current_path_string);

        last_state.wave_path(&options, &mut current_path_string);

        for state in state_iter {
            PathState::transition(&last_state, &state, &options, &mut current_path_string);
            state.wave_path(&options, &mut current_path_string);

            last_state = state;
        }

        last_state.end(&options, &mut current_path_string);

        AssembledWavePath {
            segments: current_path_string.segments,
        }
    }

    pub fn render(&self) -> AssembledWavePath {
        self.render_with_options(&WaveDimension::default())
    }
}

impl AssembledWavePath {
    pub fn segments(&self) -> &[WavePathSegment] {
        &self.segments
    }
}

impl PathCommand {
    pub fn has_no_stroke(&self) -> bool {
        match self {
            Self::LineHorizontal(..) | Self::Line(..) => false,
            Self::LineVerticalNoStroke(..) => true,
        }
    }
}

impl PathData {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            current_x: x,
            current_y: y,

            start_x: x,
            start_y: y,

            is_fully_stroked: true,
            actions: Vec::new(),
        }
    }

    pub fn horizontal_line(&mut self, dx: i32) {
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

    pub fn line(&mut self, dx: i32, dy: i32) {
        self.current_x += dx;
        self.current_y += dy;

        self.actions.push(PathCommand::Line(dx, dy));
    }

    fn vertical_line_no_stroke(&mut self, dy: i32) {
        self.current_y += dy;
        self.is_fully_stroked = false;
        self.actions.push(PathCommand::LineVerticalNoStroke(dy));
    }

    pub fn take_and_restart_at(&mut self, x: i32, y: i32) -> PathData {
        let taken = PathData {
            current_x: self.current_x,
            current_y: self.current_y,

            start_x: self.start_x,
            start_y: self.start_y,

            is_fully_stroked: self.is_fully_stroked,
            actions: self.actions.split_off(0),
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
}

impl PathString {
    fn new(x: i32, y: i32) -> Self {
        Self {
            forward: PathData::new(x, y),
            backward: PathData::new(0, 0),
            segments: Vec::new(),
        }
    }

    fn commit_with_back_line(&mut self, number: usize) {
        let segment_start_x = self.forward.start_x;
        let segment_start_y = self.forward.start_y;

        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        let is_fully_stroked = self.forward.is_fully_stroked && self.backward.is_fully_stroked;

        // TODO: Optimize this.
        for action in self.backward.take_and_restart_at(0, 0).actions.into_iter().rev() {
            self.forward.actions.push(action);
        }

        let close_status = PathSegmentCloseStatus::Encased(PathSegmentEncasement {
            data_index: number,
            is_fully_stroked,
        });

        let actions = self.forward.take_and_restart_at(start_x, start_y).actions;

        self.segments.push(WavePathSegment {
            x: segment_start_x,
            y: segment_start_y,

            close_status,
            actions,
        });
    }

    fn commit_without_back_line(&mut self) {
        let segment_start_x = self.forward.start_x;
        let segment_start_y = self.forward.start_y;

        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        let close_status = PathSegmentCloseStatus::Open;
        let actions = self.forward.take_and_restart_at(start_x, start_y).actions;

        self.segments.push(WavePathSegment {
            x: segment_start_x,
            y: segment_start_y,

            close_status,
            actions,
        });
    }
}

impl PathState {
    fn transition(&self, next: &Self, dimensions: &WaveDimension, path_string: &mut PathString) {
        let t = i32::from(dimensions.transition_offset);
        let h = i32::from(dimensions.wave_height);

        use PathState::*;

        match (self, next) {
            (Top, Top) | (Bottom, Bottom) => path_string.forward.horizontal_line(t * 2),
            (Box(a), Box(b)) if a == b => {
                path_string.forward.horizontal_line(t * 2);
                path_string.backward.horizontal_line(-t * 2);
            }
            (Box(lhs), Box(_)) => {
                path_string.forward.line(t, h / 2);
                path_string.backward.line(-t, h / 2);

                path_string.commit_with_back_line(*lhs);

                path_string.forward.line(t, -h / 2);
                path_string.backward.line(-t, -h / 2);
            }
            (Top, Bottom) => path_string.forward.line(t * 2, h),
            (Bottom, Top) => path_string.forward.line(t * 2, -h),
            (Bottom, Box(_)) => {
                path_string.forward.horizontal_line(t);

                path_string.commit_without_back_line();

                path_string.forward.line(t, -h);
                path_string.backward.horizontal_line(-t);
            }
            (Top, Box(_)) => {
                path_string.forward.horizontal_line(t);

                path_string.commit_without_back_line();

                path_string.forward.horizontal_line(t);
                path_string.backward.line(-t, -h);
            }
            (Box(lhs), Top) => {
                path_string.forward.horizontal_line(t);
                path_string.backward.line(-t, h);

                path_string.commit_with_back_line(*lhs);

                path_string.forward.horizontal_line(t);
            }
            (Box(lhs), Bottom) => {
                path_string.forward.line(t, h);
                path_string.backward.horizontal_line(-t);

                path_string.commit_with_back_line(*lhs);

                path_string.forward.horizontal_line(t);
            }
        }
    }

    fn wave_path(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        let t = i32::from(dimensions.transition_offset);
        let w = i32::from(dimensions.cycle_width);

        match self {
            Self::Top | Self::Bottom => path_string.forward.horizontal_line(w - t * 2),
            Self::Box(_) => {
                path_string.forward.horizontal_line(w - t * 2);
                path_string.backward.horizontal_line(t * 2 - w);
            }
        }
    }

    fn begin(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        let t = i32::from(dimensions.transition_offset);
        let h = i32::from(dimensions.wave_height);

        match self {
            Self::Top => path_string.forward.horizontal_line(t),
            Self::Bottom => {
                path_string.forward.restart_move_to(0, h);
                path_string.forward.horizontal_line(t);
            }
            Self::Box(_) => {
                path_string.forward.horizontal_line(t);
                path_string.backward.vertical_line_no_stroke(-h);
                path_string.backward.horizontal_line(-t);
            }
        }
    }

    fn end(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        let t = i32::from(dimensions.transition_offset);
        let h = i32::from(dimensions.wave_height);

        match self {
            Self::Top | Self::Bottom => {
                path_string.forward.horizontal_line(t);
                path_string.commit_without_back_line();
            }
            Self::Box(lhs) => {
                path_string.forward.horizontal_line(t);
                path_string.forward.vertical_line_no_stroke(h);
                path_string.backward.horizontal_line(-t);
                path_string.commit_with_back_line(*lhs);
            }
        }
    }
}
