use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InCycleOffset {
    #[default]
    Begin,
    Quarter,
    Half,
    ThreeQuarter,
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

        Ok(Self { index, in_offset })
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
    #[inline]
    pub fn new(index: u32, in_offset: InCycleOffset) -> Self {
        Self { index, in_offset }
    }

    #[inline]
    pub fn new_rounded(index: u32) -> Self {
        Self {
            index,
            in_offset: InCycleOffset::Begin,
        }
    }

    pub fn cycle_width(self) -> u32 {
        self.index
            + match self.in_offset {
                InCycleOffset::Begin => 0,
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
