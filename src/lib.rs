use std::str::FromStr;

pub struct Wave {
    pub name: String,
    pub cycles: Cycles,
}

pub struct Figure(pub Vec<Wave>);

pub struct Cycles(pub Vec<CycleData>);

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

pub struct WavePath(Vec<PathState>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathState {
    Top,
    Bottom,
    Box(usize),
}

impl PathState {
    fn has_back_line(&self) -> bool {
        matches!(self, Self::Box(..))
    }
}

const CYCLE_HEIGHT: i32 = 14;
const CYCLE_WIDTH: i32 = 20;
const TRANSITION_OFFSET: i32 = 2;

#[derive(Debug, Clone)]
pub enum PathAction {
    MoveToAbsolute(i32, i32),
    MoveToRelative(i32, i32),
    HLineToRelative(i32),
    LineToRelative(i32, i32),
    Close,
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

    fn scale(&mut self, f: i32) {
        match self {
            Self::MoveToAbsolute(ref mut x, ref mut y)
            | Self::MoveToRelative(ref mut x, ref mut y)
            | Self::LineToRelative(ref mut x, ref mut y) => {
                *x *= f;
                *y *= f;
            }
            Self::HLineToRelative(ref mut x) => *x *= f,
            Self::Close => {}
        }
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

#[derive(Debug, Clone)]
pub struct PathD {
    current_x: i32,
    current_y: i32,
    actions: Vec<PathAction>,
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

        if let Some(PathAction::HLineToRelative(ref mut last_dx)) = self.actions.last_mut() {
            *last_dx += dx;
        } else {
            self.actions.push(PathAction::h(dx));
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

    pub fn append(&mut self, d: &mut PathD) {
        self.actions.append(&mut d.actions)
    }
}

pub struct PathString {
    forward: PathD,
    backward: PathD,
    groups: Vec<(PathD, Option<usize>)>,
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
    fn transition(&self, next: &Self, path_string: &mut PathString) {
        use PathState::*;

        match (self, next) {
            (Top, Top) | (Bottom, Bottom) => path_string
                .forward
                .horizontal_line_to_relative(TRANSITION_OFFSET * 2),
            (Box(a), Box(b)) if a == b => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET * 2);
                path_string
                    .backward
                    .horizontal_line_to_relative(TRANSITION_OFFSET * -2);
            }
            (Box(lhs), Box(_)) => {
                path_string
                    .forward
                    .line_to_relative(TRANSITION_OFFSET, CYCLE_HEIGHT / 2);
                path_string
                    .backward
                    .line_to_relative(-1 * TRANSITION_OFFSET, CYCLE_HEIGHT / 2);

                path_string.commit_with_back_line(*lhs);

                path_string
                    .forward
                    .line_to_relative(TRANSITION_OFFSET, -1 * CYCLE_HEIGHT / 2);
                path_string
                    .backward
                    .line_to_relative(-1 * TRANSITION_OFFSET, -1 * CYCLE_HEIGHT / 2);
            }
            (Top, Bottom) => path_string
                .forward
                .line_to_relative(TRANSITION_OFFSET * 2, CYCLE_HEIGHT),
            (Bottom, Top) => path_string
                .forward
                .line_to_relative(TRANSITION_OFFSET * 2, -1 * CYCLE_HEIGHT),
            (Bottom, Box(_)) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);

                path_string.commit_without_back_line();

                path_string
                    .forward
                    .line_to_relative(TRANSITION_OFFSET, -1 * CYCLE_HEIGHT);
                path_string
                    .backward
                    .horizontal_line_to_relative(TRANSITION_OFFSET * -1);
            }
            (Top, Box(_)) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);

                path_string.commit_without_back_line();

                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
                path_string
                    .backward
                    .line_to_relative(TRANSITION_OFFSET * -1, -1 * CYCLE_HEIGHT);
            }
            (Box(lhs), Top) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
                path_string
                    .backward
                    .line_to_relative(TRANSITION_OFFSET * -1, CYCLE_HEIGHT);

                path_string.commit_with_back_line(*lhs);

                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
            }
            (Box(lhs), Bottom) => {
                path_string
                    .forward
                    .line_to_relative(TRANSITION_OFFSET * -1, 1 * CYCLE_HEIGHT);
                path_string
                    .backward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);

                path_string.commit_with_back_line(*lhs);

                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
            }
        }
    }

    fn wave_path(&self, path_string: &mut PathString) {
        match self {
            Self::Top | Self::Bottom => path_string
                .forward
                .horizontal_line_to_relative(CYCLE_WIDTH - TRANSITION_OFFSET * 2),
            Self::Box(_) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(CYCLE_WIDTH - TRANSITION_OFFSET * 2);
                path_string
                    .backward
                    .horizontal_line_to_relative(-1 * (CYCLE_WIDTH - TRANSITION_OFFSET * 2));
            }
        }
    }

    fn begin(&self, path_string: &mut PathString) {
        match self {
            Self::Top => path_string
                .forward
                .horizontal_line_to_relative(TRANSITION_OFFSET),
            Self::Bottom => {
                path_string.forward.move_to_relative(0, CYCLE_HEIGHT);
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
            }
            Self::Box(_) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
                path_string
                    .backward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
            }
        }
    }

    fn end(&self, path_string: &mut PathString) {
        match self {
            Self::Top | Self::Bottom => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET);
                path_string.commit_without_back_line();
            }
            Self::Box(lhs) => {
                path_string
                    .forward
                    .horizontal_line_to_relative(TRANSITION_OFFSET * 2);
                path_string
                    .backward
                    .horizontal_line_to_relative(TRANSITION_OFFSET * -2);
                path_string.commit_with_back_line(*lhs);
            }
        }
    }
}

impl WavePath {
    fn to_paths(&self, x: i32, y: i32) -> Vec<(PathD, Option<usize>)> {
        let mut state_iter = self.0.iter();
        let Some(mut last_state) = state_iter.next() else {
            return Vec::new();
        };

        let mut current_path_string = PathString::new(x, y);

        last_state.begin(&mut current_path_string);

        last_state.wave_path(&mut current_path_string);

        for state in state_iter {
            PathState::transition(&last_state, &state, &mut current_path_string);
            state.wave_path(&mut current_path_string);

            last_state = state;
        }

        last_state.end(&mut current_path_string);

        current_path_string.groups
    }
}

impl Wave {
    pub fn to_svg(&self, writer: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        let wave_path = WavePath(
            self.cycles
                .0
                .clone()
                .into_iter()
                .map(|s| match s {
                    CycleData::Top => PathState::Top,
                    CycleData::Bottom => PathState::Bottom,
                    CycleData::Box(usize) => PathState::Box(usize),
                })
                .collect(),
        );


        for (path, container_number) in wave_path.to_paths(20, 20).into_iter() {
            let fill = match container_number {
                Some(0) => "#ff4040",
                Some(1) => "#5499C7",
                Some(2) => "#58D68D",
                Some(3) => "#A569BD",
                _ => "none",
            };
            write!(writer, r##"<path fill="{fill}" d=""##)?;
            for action in path.actions {
                write!(writer, "{action}")?;
            }
            write!(writer, r##"" stroke-width="1" stroke="#000"/>"##)?;
        }

        Ok(())
    }
}

impl Figure {
    pub fn to_svg(&self, writer: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">"#
        )?;


        if let Some(num_cycles) = self.0.iter().map(|w| w.cycles.0.len()).max() {
            write!(
                writer,
                r##"<defs><g id="cl"><path fill="none" d="M0,0v{height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC" /></g></defs>"##,
                height = 20 + ((self.0.len() as i32) * CYCLE_HEIGHT * 2),
            )?;

            write!(writer, r##"<g transform="translate(0,{y})">"##, y = 10)?;
            for i in 0..=num_cycles {
                write!(
                    writer,
                    r##"<use transform="translate({x})" xlink:href="#cl" />"##,
                    x = 20 + (i as i32) * CYCLE_WIDTH,
                )?;
            }
            write!(writer, r##"</g>"##)?;

            for (i, wave) in self.0.iter().enumerate() {
                write!(writer, r##"<g transform="translate(0,{y})">"##, y = (i as i32) * CYCLE_WIDTH * 2)?;

                wave.to_svg(writer)?;

                write!(writer, r##"</g>"##)?;
            }
        }

        write!(writer, "</svg>")?;

        Ok(())
    }
}
