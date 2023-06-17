use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

/// The cycle offset within a single-clock cycle.
///
/// At the moment a cycle in divided into 4 quarters and you 
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InCycleOffset {
    /// 0/4
    #[default]
    Begin,
    /// 1/4
    Quarter,
    /// 2/4
    Half,
    /// 3/4
    ThreeQuarter,
}

/// The cycle offset within multiple clock cycles. This is precise up until a quarter of a cycle.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CycleOffset {
    index: u32,
    in_offset: InCycleOffset,
}

impl PartialOrd for CycleOffset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match self.index.cmp(&other.index) {
            Ordering::Equal => self.in_offset.cmp(&other.in_offset),
            ord => ord,
        })
    }
}

impl Ord for CycleOffset {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.index.cmp(&other.index) {
            Ordering::Equal => self.in_offset.cmp(&other.in_offset),
            ord => ord,
        }
    }
}

impl From<f32> for CycleOffset {
    fn from(value: f32) -> Self {
        if value.is_subnormal() {
            return Self::default();
        }

        let in_offset = match (value.fract() * 4.).round() as u32 {
            4 | 0 => InCycleOffset::Begin,
            1 => InCycleOffset::Quarter,
            2 => InCycleOffset::Half,
            _ => InCycleOffset::ThreeQuarter,
        };
        let index = if value.fract() > 0.75 && in_offset == InCycleOffset::Begin {
            value.ceil()
        } else {
            value.floor()
        } as u32;

        Self { index, in_offset }
    }
}

impl From<f64> for CycleOffset {
    fn from(value: f64) -> Self {
        if value.is_subnormal() {
            return Self::default();
        }

        let in_offset = match ((value * 4.).round() % 4.) as u32 {
            0 => InCycleOffset::Begin,
            1 => InCycleOffset::Quarter,
            2 => InCycleOffset::Half,
            _ => InCycleOffset::ThreeQuarter,
        };
        let index = if value.fract() > 0.75 && in_offset == InCycleOffset::Begin {
            value.ceil()
        } else {
            value.floor()
        } as u32;

        Self { index, in_offset }
    }
}

impl Add for CycleOffset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use InCycleOffset::*;

        let (in_offset, carry) = match (self.in_offset, rhs.in_offset) {
            (x, Begin) | (Begin, x) => (x, 0),
            (Quarter, Quarter) => (Half, 0),
            (Quarter, Half) | (Half, Quarter) => (ThreeQuarter, 0),
            (Half, Half)
            | (Quarter, ThreeQuarter)
            | (ThreeQuarter, Quarter) => (Begin, 1),
            (Half, ThreeQuarter) | (ThreeQuarter, Half) => (Quarter, 1),
            (ThreeQuarter, ThreeQuarter) => (Half, 1),
        };
        let index = self.index + rhs.index + carry;

        Self { index, in_offset }
    }
}

impl AddAssign for CycleOffset {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl CycleOffset {
    /// Create a new [`CycleOffset`] with a specific cycle cycle `index` and an in cycle cycle
    /// offset `in_offset`
    #[inline]
    pub fn new(index: u32, in_offset: InCycleOffset) -> Self {
        Self { index, in_offset }
    }

    /// Create a new [`CycleOffset`] that is rounded to an specific cycle cycle `index`
    #[inline]
    pub fn new_rounded(index: u32) -> Self {
        Self {
            index,
            in_offset: InCycleOffset::Begin,
        }
    }

    /// Get the a ceiled value of the number of cycles that the [`CycleOffset`] incorperates.
    pub fn ceil_num_cycles(self) -> u32 {
        self.index
            + match self.in_offset {
                InCycleOffset::Begin => 0,
                _ => 1,
            }
    }

    /// Get the `index` of the [`CycleOffset`]
    #[inline]
    pub fn cycle_index(self) -> u32 {
        self.index
    }

    /// Get the in cycle offset of the [`CycleOffset`]
    #[inline]
    pub fn in_cycle_offset(self) -> InCycleOffset {
        self.in_offset
    }

    /// Get the width knowning that a cycle cycle is `width` units wide.
    #[inline]
    pub fn width_offset(self, width: u32) -> u32 {
        width * self.index + self.in_offset.width_offset(width)
    }

    /// Half the [`CycleOffset`]
    pub fn half(&self) -> CycleOffset {
        use InCycleOffset::*;

        if self.index % 2 == 0 {
            Self::new(self.index / 2, match self.in_offset {
                Begin | Quarter => Begin,
                Half | ThreeQuarter => Quarter,
            })
        } else {
            Self::new(self.index / 2, match self.in_offset {
                Begin | Quarter => Half,
                Half | ThreeQuarter => ThreeQuarter,
            })
        }
    }
}

impl InCycleOffset {
    /// Get the width knowning that a cycle cycle is `width` units wide.
    pub fn width_offset(self, width: u32) -> u32 {
        let w = f64::from(width);

        match self {
            Self::Begin => 0,
            Self::Quarter => (w * 0.25).round() as u32,
            Self::Half => (w * 0.5).round() as u32,
            Self::ThreeQuarter => (w * 0.75).round() as u32,
        }
    }
}
