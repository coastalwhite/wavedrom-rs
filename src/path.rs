pub struct WavePath(Vec<PathState>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathState {
    Top,
    Bottom,
    Box(usize),
}

#[derive(Debug, Clone)]
pub enum PathAction {
    MoveToAbsolute(i32, i32),
    MoveToRelative(i32, i32),
    HLineToRelative(i32),
    LineToRelative(i32, i32),
    Close,
}

#[derive(Debug, Clone)]
pub struct PathD {
    current_x: i32,
    current_y: i32,
    pub(crate) actions: Vec<PathAction>,
}

#[derive(Debug)]
pub struct PathString {
    forward: PathD,
    backward: PathD,
    groups: Vec<(PathD, Option<usize>)>,
}

#[derive(Debug, Clone)]
pub struct WaveDimension {
    wave_height: i32,
    cycle_width: i32,
    transition_offset: i32,
}

impl WaveDimension {
    pub(crate) fn wave_height_f64(&self) -> f64 {
        self.wave_height.into()
    }

    pub(crate) fn cycle_width_f64(&self) -> f64 {
        self.cycle_width.into()
    }
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

impl WavePath {
    #[inline]
    pub fn new(states: Vec<PathState>) -> Self {
        Self(states)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn to_paths(&self, dimensions: &WaveDimension) -> Vec<(PathD, Option<usize>)> {
        let mut state_iter = self.0.iter();
        let Some(mut last_state) = state_iter.next() else {
            return Vec::new();
        };

        let mut current_path_string = PathString::new(0, 0);

        last_state.begin(dimensions, &mut current_path_string);

        last_state.wave_path(dimensions, &mut current_path_string);

        for state in state_iter {
            PathState::transition(&last_state, &state, dimensions, &mut current_path_string);
            state.wave_path(dimensions, &mut current_path_string);

            last_state = state;
        }

        last_state.end(dimensions, &mut current_path_string);

        current_path_string.groups
    }
}

impl PathAction {
    const fn h(dx: i32) -> Self {
        Self::HLineToRelative(dx)
    }

    const fn l(dx: i32, dy: i32) -> Self {
        Self::LineToRelative(dx, dy)
    }

    const fn m(dx: i32, dy: i32) -> Self {
        Self::MoveToRelative(dx, dy)
    }

    const fn z() -> Self {
        Self::Close
    }
}

impl std::fmt::Display for PathAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoveToAbsolute(x, y) => write!(f, "M{x},{y}"),
            Self::MoveToRelative(dx, dy) => write!(f, "m{dx},{dy}"),
            Self::HLineToRelative(dx) => write!(f, "h{dx}"),
            Self::LineToRelative(dx, dy) => write!(f, "l{dx},{dy}"),
            Self::Close => write!(f, "z"),
        }
    }
}

impl PathD {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            current_x: x,
            current_y: y,
            actions: vec![PathAction::MoveToAbsolute(x, y)],
        }
    }

    pub fn new_without_position() -> Self {
        Self {
            current_x: 0,
            current_y: 0,
            actions: Vec::new(),
        }
    }

    pub fn horizontal_line_to_relative(&mut self, dx: i32) {
        self.current_x += dx;

        match self.actions.last_mut() {
            Some(PathAction::HLineToRelative(ref mut last_dx))
                if dx.signum() == last_dx.signum() =>
            {
                *last_dx += dx
            }
            _ => self.actions.push(PathAction::h(dx)),
        }
    }

    pub fn line_to_relative(&mut self, dx: i32, dy: i32) {
        self.current_x += dx;
        self.current_y += dy;

        self.actions.push(PathAction::l(dx, dy));
    }

    pub fn move_to_relative(&mut self, dx: i32, dy: i32) {
        self.current_x += dx;
        self.current_y += dy;

        match self.actions.last_mut() {
            Some(
                PathAction::MoveToAbsolute(ref mut x, ref mut y)
                | PathAction::MoveToRelative(ref mut x, ref mut y),
            ) => {
                *x += dx;
                *y += dy;
            }
            _ => self.actions.push(PathAction::m(dx, dy)),
        }
    }

    pub fn close(&mut self) {
        self.current_x = 0;
        self.current_y = 0;

        self.actions.push(PathAction::z());
    }

    pub fn take(&mut self) -> PathD {
        let taken = PathD {
            current_x: self.current_x,
            current_y: self.current_y,
            actions: self.actions.split_off(0),
        };

        self.current_x = 0;
        self.current_y = 0;

        taken
    }
}

impl PathString {
    fn new(x: i32, y: i32) -> Self {
        Self {
            forward: PathD::new(x, y),
            backward: PathD::new_without_position(),
            groups: Vec::new(),
        }
    }

    fn commit_with_back_line(&mut self, number: usize) {
        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        // TODO: Optimize this.
        for action in self.backward.take().actions.into_iter().rev() {
            self.forward.actions.push(action);
        }

        self.forward.close();

        self.groups.push((self.forward.take(), Some(number)));

        self.forward.move_to_relative(start_x, start_y)
    }

    fn commit_without_back_line(&mut self) {
        let start_x = self.forward.current_x;
        let start_y = self.forward.current_y;

        self.groups.push((self.forward.take(), None));

        self.forward.move_to_relative(start_x, start_y)
    }
}

impl PathState {
    fn transition(&self, next: &Self, dimensions: &WaveDimension, path_string: &mut PathString) {
        use PathState::*;

        match (self, next) {
            (Top, Top) | (Bottom, Bottom) => path_string
                .forward
                .horizontal_line_to_relative(dimensions.transition_offset * 2),
            (Box(a), Box(b)) if a == b => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset * 2);
                path_string
                    .backward
                    .horizontal_line_to_relative(dimensions.transition_offset * -2);
            }
            (Box(lhs), Box(_)) => {
                path_string
                    .forward
                    .line_to_relative(dimensions.transition_offset, dimensions.wave_height / 2);
                path_string.backward.line_to_relative(
                    -1 * dimensions.transition_offset,
                    dimensions.wave_height / 2,
                );

                path_string.commit_with_back_line(*lhs);

                path_string.forward.line_to_relative(
                    dimensions.transition_offset,
                    -1 * dimensions.wave_height / 2,
                );
                path_string.backward.line_to_relative(
                    -1 * dimensions.transition_offset,
                    -1 * dimensions.wave_height / 2,
                );
            }
            (Top, Bottom) => path_string
                .forward
                .line_to_relative(dimensions.transition_offset * 2, dimensions.wave_height),
            (Bottom, Top) => path_string.forward.line_to_relative(
                dimensions.transition_offset * 2,
                -1 * dimensions.wave_height,
            ),
            (Bottom, Box(_)) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);

                path_string.commit_without_back_line();

                path_string
                    .forward
                    .line_to_relative(dimensions.transition_offset, -1 * dimensions.wave_height);
                path_string
                    .backward
                    .horizontal_line_to_relative(dimensions.transition_offset * -1);
            }
            (Top, Box(_)) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);

                path_string.commit_without_back_line();

                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
                path_string.backward.line_to_relative(
                    dimensions.transition_offset * -1,
                    -1 * dimensions.wave_height,
                );
            }
            (Box(lhs), Top) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
                path_string
                    .backward
                    .line_to_relative(dimensions.transition_offset * -1, dimensions.wave_height);

                path_string.commit_with_back_line(*lhs);

                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
            }
            (Box(lhs), Bottom) => {
                path_string.forward.line_to_relative(
                    dimensions.transition_offset * -1,
                    1 * dimensions.wave_height,
                );
                path_string
                    .backward
                    .horizontal_line_to_relative(dimensions.transition_offset);

                path_string.commit_with_back_line(*lhs);

                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
            }
        }
    }

    fn wave_path(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        match self {
            Self::Top | Self::Bottom => path_string.forward.horizontal_line_to_relative(
                dimensions.cycle_width - dimensions.transition_offset * 2,
            ),
            Self::Box(_) => {
                path_string.forward.horizontal_line_to_relative(
                    dimensions.cycle_width - dimensions.transition_offset * 2,
                );
                path_string.backward.horizontal_line_to_relative(
                    -1 * (dimensions.cycle_width - dimensions.transition_offset * 2),
                );
            }
        }
    }

    fn begin(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        match self {
            Self::Top => path_string
                .forward
                .horizontal_line_to_relative(dimensions.transition_offset),
            Self::Bottom => {
                path_string
                    .forward
                    .move_to_relative(0, dimensions.wave_height);
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
            }
            Self::Box(_) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
                path_string
                    .backward
                    .horizontal_line_to_relative(dimensions.transition_offset);
            }
        }
    }

    fn end(&self, dimensions: &WaveDimension, path_string: &mut PathString) {
        match self {
            Self::Top | Self::Bottom => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset);
                path_string.commit_without_back_line();
            }
            Self::Box(lhs) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(dimensions.transition_offset * 2);
                path_string
                    .backward
                    .horizontal_line_to_relative(dimensions.transition_offset * -2);
                path_string.commit_with_back_line(*lhs);
            }
        }
    }
}
