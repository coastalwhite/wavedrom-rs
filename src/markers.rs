//! A collection of markers that get overlayed onto the signal diagram.

use crate::CycleOffset;

/// A marker for a group of [`AssembledLine`][crate::AssembledLine]s.
///
/// This is usually displayed on the left of the signal and can group any sequential lines
/// `start..end`. It can include a potential label. There is also a depth value for when groups are
/// nested. The depth value starts from `0`.
#[derive(Debug, Clone)]
pub struct GroupMarker<'a> {
    start: u32,
    end: u32,
    label: Option<&'a str>,
    depth: u32,
}

/// The edge of a clock cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockEdge {
    /// Low to High to Low.
    Positive,
    /// High to Low to High.
    Negative,
}

/// The enumeration above or below signals indicating the number of the current cycle.
///
/// The enumeration starts at `start` with steps of `every`. The marker contains `start,
/// start+every, start+2*every, ...` until the diagram cycle count is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleEnumerationMarker {
    start: u32,
    every: u32,
}

/// An arrow illustrating where a clock edge is happening.
#[derive(Debug, Clone)]
pub struct ClockEdgeMarker {
    at: CycleOffset,
    edge: ClockEdge,
}

impl<'a> GroupMarker<'a> {
    /// Create a new [`GroupMarker`] capturing lines `start..end`.
    #[inline]
    pub fn new(start: u32, end: u32, label: Option<&'a str>, depth: u32) -> Self {
        Self {
            start,
            end,
            label,
            depth,
        }
    }

    /// Returns amount of lines captured by the [`GroupMarker`].
    #[inline]
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Returns whether no groups are captured by a [`GroupMarker`].
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the optional label of a [`GroupMarker`].
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.label
    }

    /// Returns the nested depth of a [`GroupMarker`] starting from `0`.
    #[inline]
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns the `start` value of the [`GroupMarker`] that captures `start..end`.
    #[inline]
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Returns the `end` value of the [`GroupMarker`] that captures `start..end`.
    #[inline]
    pub fn end(&self) -> u32 {
        self.end
    }
}

impl ClockEdge {
    /// Returns `true` if the [`ClockEdge`] is a [`ClockEdge::Positive`] value.
    #[inline]
    pub fn is_positive(self) -> bool {
        self == Self::Positive
    }

    /// Returns `true` if the [`ClockEdge`] is a [`ClockEdge::Negative`] value.
    #[inline]
    pub fn is_negative(self) -> bool {
        self == Self::Negative
    }
}

impl CycleEnumerationMarker {
    /// Create a new [`CycleEnumerationMarker`] where the enumeration starts at `start` and
    /// `start, start+every, start+2*every, ..` are numbered.
    ///
    /// To number all the cycles, utilize a value of `every` equal to `0` or `1`.
    #[inline]
    pub fn new(start: u32, every: u32) -> Self {
        Self { start, every: every.max(1) }
    }

    /// Return the cycle number for the first numbered cycle.
    #[inline]
    pub fn start(self) -> u32 {
        self.start
    }

    /// Return every how manyth cycle is numbered.
    #[inline]
    pub fn every(self) -> u32 {
        self.every
    }
}

impl ClockEdgeMarker {
    /// Create a new [`ClockEdgeMarker`].
    ///
    /// The created marker is positioned in a line with an x-value that corresponds to `at` and
    /// represents the [`ClockEdge`] `edge`.
    #[inline]
    pub fn new(at: CycleOffset, edge: ClockEdge) -> Self {
        Self { at, edge }
    }

    /// Returns where in the line the [`ClockEdgeMarker`] is positioned.
    #[inline]
    pub fn at(&self) -> CycleOffset {
        self.at
    }

    /// Returns which edge the [`ClockEdgeMarker`] represents.
    #[inline]
    pub fn edge(&self) -> ClockEdge {
        self.edge
    }
}
