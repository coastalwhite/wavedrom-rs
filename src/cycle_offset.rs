use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InCycleOffset {
    #[default]
    OffsetNone,
    OffsetQuarter,
    OffsetHalf,
    Offset3Quarter,
}

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

impl TryFrom<f32> for CycleOffset {
    type Error = ();

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_subnormal() {
            return Err(());
        }

        let in_offset = match (value.fract() * 4.).round() as u32 {
            4 | 0 => InCycleOffset::OffsetNone,
            1 => InCycleOffset::OffsetQuarter,
            2 => InCycleOffset::OffsetHalf,
            _ => InCycleOffset::Offset3Quarter,
        };
        let index = if value.fract() > 0.75 && in_offset == InCycleOffset::OffsetNone {
            value.ceil()
        } else {
            value.floor()
        } as u32;

        Ok(Self { index, in_offset })
    }
}

impl TryFrom<f64> for CycleOffset {
    type Error = ();

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_subnormal() {
            return Err(());
        }

        let in_offset = match ((value * 4.).round() % 4.) as u32 {
            0 => InCycleOffset::OffsetNone,
            1 => InCycleOffset::OffsetQuarter,
            2 => InCycleOffset::OffsetHalf,
            _ => InCycleOffset::Offset3Quarter,
        };
        let index = if value.fract() > 0.75 && in_offset == InCycleOffset::OffsetNone {
            value.ceil()
        } else {
            value.floor()
        } as u32;

        Ok(Self { index, in_offset })
    }
}

impl Add for CycleOffset {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use InCycleOffset::*;

        let (in_offset, carry) = match (self.in_offset, rhs.in_offset) {
            (x, OffsetNone) | (OffsetNone, x) => (x, 0),
            (OffsetQuarter, OffsetQuarter) => (OffsetHalf, 0),
            (OffsetQuarter, OffsetHalf) | (OffsetHalf, OffsetQuarter) => (Offset3Quarter, 0),
            (OffsetHalf, OffsetHalf)
            | (OffsetQuarter, Offset3Quarter)
            | (Offset3Quarter, OffsetQuarter) => (OffsetNone, 1),
            (OffsetHalf, Offset3Quarter) | (Offset3Quarter, OffsetHalf) => (OffsetQuarter, 1),
            (Offset3Quarter, Offset3Quarter) => (OffsetHalf, 1),
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
    #[inline]
    pub fn new(index: u32, in_offset: InCycleOffset) -> Self {
        Self { index, in_offset }
    }

    #[inline]
    pub fn new_rounded(index: u32) -> Self {
        Self {
            index,
            in_offset: InCycleOffset::OffsetNone,
        }
    }

    pub fn cycle_width(self) -> u32 {
        self.index
            + match self.in_offset {
                InCycleOffset::OffsetNone => 0,
                _ => 1,
            }
    }

    #[inline]
    pub fn cycle_index(self) -> u32 {
        self.index
    }

    #[inline]
    pub fn in_cycle_offset(self) -> InCycleOffset {
        self.in_offset
    }

    #[inline]
    pub fn width_offset(self, width: u32) -> u32 {
        width * self.index + self.in_offset.width_offset(width)
    }
}

impl InCycleOffset {
    pub fn width_offset(self, width: u32) -> u32 {
        let w = f64::from(width);

        match self {
            Self::OffsetNone => 0,
            Self::OffsetQuarter => (w * 0.25).round() as u32,
            Self::OffsetHalf => (w * 0.5).round() as u32,
            Self::Offset3Quarter => (w * 0.75).round() as u32,
        }
    }
}
