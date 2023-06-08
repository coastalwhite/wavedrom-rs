use crate::CycleOffset;

#[derive(Debug, Clone)]
pub struct GroupMarker<'a> {
    start: u32,
    end: u32,
    label: Option<&'a str>,
    depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockEdge {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleMarker {
    start: u32,
    every: u32,
}

#[derive(Debug, Clone)]
pub struct ClockEdgeMarker {
    at: CycleOffset,
    edge: ClockEdge,
}

impl<'a> GroupMarker<'a> {
    #[inline]
    pub fn new(start: u32, end: u32, label: Option<&'a str>, depth: u32) -> Self {
        Self {
            start,
            end,
            label,
            depth,
        }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label
    }

    #[inline]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    #[inline]
    pub fn start(&self) -> u32 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> u32 {
        self.end
    }
}

impl ClockEdge {
    #[inline]
    pub fn is_positive(self) -> bool {
        self == Self::Positive
    }

    #[inline]
    pub fn is_negative(self) -> bool {
        self == Self::Negative
    }
}

impl CycleMarker {
    #[inline]
    pub fn new(start: u32, every: u32) -> Self {
        Self { start, every }
    }

    #[inline]
    pub fn start(self) -> u32 {
        self.start
    }

    #[inline]
    pub fn every(self) -> u32 {
        self.every
    }
}

impl ClockEdgeMarker {
    #[inline]
    pub fn new(at: CycleOffset, edge: ClockEdge) -> Self {
        Self { at, edge }
    }

    #[inline]
    pub fn at(&self) -> CycleOffset {
        self.at
    }

    #[inline]
    pub fn edge(&self) -> ClockEdge {
        self.edge
    }
}
