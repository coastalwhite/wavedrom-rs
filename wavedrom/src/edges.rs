//! Edges or Arrows define a set of markers and edge lines that can be put over a diagram to
//! indicate properties.
//!
//! An edge line is between 2 nodes which are identified by a character. If the character is and
//! uppercase ASCII character then is is not displayed on the diagram otherwise it is also shown on
//! the diagram.
//!
//! There are several types of edges, a full overview can be seen in the [wavedrom-rs book][book].
//! Here they are represented with the [`EdgeVariant`] structure.
//!
//! In [WaveJson][crate::wavejson] an edge is given by a string that defined under the `edge`
//! property array at the root JSON level. The edge there given in the following order: `<start
//! node><edge identifier><end node> [label]`. The label is text that is put on the middle of the
//! edge.
//!
//! [book]: https://coastalwhite.github.io/wavedrom-rs

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use crate::{CycleOffset, Signal};

/// A set of edge markers. Both the edge lines and the text_nodes.
#[derive(Debug, Clone)]
pub struct LineEdgeMarkers<'a> {
    lines: Vec<LineEdge<'a>>,
    text_nodes: Vec<LineEdgeText>,
}

/// A edge from a start node to an end node
#[derive(Debug, Clone)]
pub struct LineEdge<'a> {
    from: InSignalPosition,
    from_marker: Option<char>,
    to: InSignalPosition,
    to_marker: Option<char>,
    text: Option<Cow<'a, str>>,
    variant: EdgeVariant,
}

/// The text belowing to a node
#[derive(Debug, Clone)]
pub struct LineEdgeText {
    at: InSignalPosition,
    text: char,
}

/// A position in the signal schema. Containing both a `x` (cycle offset) value and a `y` (signal
/// index) value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InSignalPosition {
    x: CycleOffset,
    y: u32,
}

/// The definition for an edge
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct EdgeDefinition {
    variant: EdgeVariant,
    from: char,
    to: char,
    label: Option<String>,
}

/// A variant of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeVariant {
    /// A smooth / curved edge variant
    Spline(SplineEdgeVariant),
    /// A sharp edge variant
    Sharp(SharpEdgeVariant),
}

/// A variant of a smooth / curved edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplineEdgeVariant {
    /// Spline edge that is points to the start and end node horizontally. Edge identifier is `~`.
    BothHorizontal(EdgeArrowType),
    /// Spline edge that is points to the start horizontally. The end node is pointed to slightly
    /// vertical. Edge identifier is `-~`.
    StartHorizontal(EdgeArrowType),
    /// Spline edge that is points to the end horizontally. The start node is pointed to slightly
    /// vertical. Edge identifier is `~-`.
    EndHorizontal(EdgeArrowType),
}

/// A variant of a sharp edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharpEdgeVariant {
    /// Sharp edge that always takes the shortest path from the start to the end node. Edge
    /// identifier is `-`.
    Straight(EdgeArrowType),
    /// Sharp edge that points to the start and the end node horizontally. Edge identifier is
    /// `-|-`.
    BothHorizontal(EdgeArrowType),
    /// Sharp edge that points to the start node horizontally and the end node mostly vertically.
    /// Edge identifier is `-|`.
    StartHorizontal(EdgeArrowType),
    /// Sharp edge that points to the end node horizontally and the start node mostly vertically.
    /// Edge identifier is `|-`.
    EndHorizontal(EdgeArrowType),
    /// Sharp edge that takes the shortest path from the start node to the end node and contains
    /// small bars at the start and end. Edge identifier is `+`.
    Cross,
}

/// Structure that defines at which sides of an [`EdgeVariant`] there are arrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeArrowType {
    /// No arrows at either start or end.
    None,
    /// Arrows at start.
    Start,
    /// Arrow at end.
    End,
    /// Both arrows at start or end.
    Both,
}

pub(crate) struct LineEdgeMarkersBuilder {
    line_number: u32,
    node_positions: HashMap<char, InSignalPosition>,
    text_nodes: Vec<LineEdgeText>,
}

impl EdgeArrowType {
    /// Create a new [`EdgeArrowType`].
    #[inline]
    fn new(has_arrow_left: bool, has_arrow_right: bool) -> Self {
        match (has_arrow_left, has_arrow_right) {
            (false, false) => Self::None,
            (true, false) => Self::Start,
            (false, true) => Self::End,
            (true, true) => Self::Both,
        }
    }
}

#[inline]
fn take_char(s: &str, c: char) -> (&str, bool) {
    if s.starts_with(c) {
        (&s[1..], true)
    } else {
        (s, false)
    }
}

#[inline]
fn take(s: &str) -> Option<(&str, char)> {
    let mut chars = s.chars();
    let c = chars.next()?;
    Some((chars.as_str(), c))
}

impl EdgeArrowType {
    /// Does the variant have an arrow at the start
    #[inline]
    pub fn has_start_arrow(self) -> bool {
        matches!(self, Self::Both | Self::Start)
    }

    /// Does the variant have an arrow at the end
    #[inline]
    pub fn has_end_arrow(self) -> bool {
        matches!(self, Self::Both | Self::End)
    }
}

impl SplineEdgeVariant {
    /// Fetch the arrow type that the [`SplineEdgeVariant`] has.
    #[inline]
    pub fn arrow_type(self) -> EdgeArrowType {
        match self {
            SplineEdgeVariant::BothHorizontal(a)
            | SplineEdgeVariant::StartHorizontal(a)
            | SplineEdgeVariant::EndHorizontal(a) => a,
        }
    }
}

impl SharpEdgeVariant {
    /// Fetch the arrow type that the [`SharpEdgeVariant`] has. The [`SharpEdgeVariant::Cross`]
    /// always has [`EdgeArrowType::None`].
    #[inline]
    pub fn arrow_type(self) -> EdgeArrowType {
        match self {
            SharpEdgeVariant::Straight(a)
            | SharpEdgeVariant::BothHorizontal(a)
            | SharpEdgeVariant::StartHorizontal(a)
            | SharpEdgeVariant::EndHorizontal(a) => a,
            SharpEdgeVariant::Cross => EdgeArrowType::None,
        }
    }
}

impl EdgeVariant {
    /// Fetch the arrow type that the [`EdgeVariant`] has. The [`SharpEdgeVariant::Cross`]
    /// always has [`EdgeArrowType::None`].
    #[inline]
    pub fn arrow_type(self) -> EdgeArrowType {
        match self {
            EdgeVariant::Spline(v) => v.arrow_type(),
            EdgeVariant::Sharp(v) => v.arrow_type(),
        }
    }

    fn consume(s: &str) -> Option<(&str, Self)> {
        let (s, has_arrow_left) = take_char(s, '<');

        match s.as_bytes() {
            [b'-', b'|', b'-', ..] => {
                let s = &s[3..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((s, Self::Sharp(SharpEdgeVariant::BothHorizontal(arrow_type))))
            }
            [b'-', b'|', ..] => {
                let s = &s[2..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((
                    s,
                    Self::Sharp(SharpEdgeVariant::StartHorizontal(arrow_type)),
                ))
            }
            [b'|', b'-', ..] => {
                let s = &s[2..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((s, Self::Sharp(SharpEdgeVariant::EndHorizontal(arrow_type))))
            }
            [b'-', b'~', ..] => {
                let s = &s[2..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((
                    s,
                    Self::Spline(SplineEdgeVariant::StartHorizontal(arrow_type)),
                ))
            }
            [b'~', b'-', ..] => {
                let s = &s[2..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((
                    s,
                    Self::Spline(SplineEdgeVariant::EndHorizontal(arrow_type)),
                ))
            }
            [b'-', ..] => {
                let s = &s[1..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((s, Self::Sharp(SharpEdgeVariant::Straight(arrow_type))))
            }
            [b'+', ..] => {
                let s = &s[1..];
                if has_arrow_left {
                    return None;
                }
                Some((s, Self::Sharp(SharpEdgeVariant::Cross)))
            }
            [b'~', ..] => {
                let s = &s[1..];
                let (s, has_arrow_right) = take_char(s, '>');
                let arrow_type = EdgeArrowType::new(has_arrow_left, has_arrow_right);
                Some((
                    s,
                    Self::Spline(SplineEdgeVariant::BothHorizontal(arrow_type)),
                ))
            }
            _ => None,
        }
    }
}

impl LineEdgeMarkersBuilder {
    pub fn new() -> Self {
        Self {
            line_number: 0,
            node_positions: HashMap::new(),
            text_nodes: Vec::new(),
        }
    }

    pub fn add_signal(&mut self, signal: &Signal) {
        let line_number = self.line_number;

        for (i, c) in signal.get_nodes().chars().enumerate() {
            if c == '.' {
                continue;
            }

            let at = InSignalPosition {
                x: signal.get_phase() + CycleOffset::new_rounded(i as u32),
                y: line_number,
            };

            self.node_positions.insert(c, at.clone());
            self.text_nodes.push(LineEdgeText { at, text: c });
        }

        self.line_number += 1;
    }

    pub fn build(mut self, edges: &[EdgeDefinition]) -> LineEdgeMarkers {
        let mut lines = Vec::new();
        let mut used_text_nodes = HashSet::new();

        for edge in edges {
            if edge.from == edge.to {
                continue;
            }

            let Some(from) = self.node_positions.get(&edge.from) else {
                continue;
            };
            let Some(to) = self.node_positions.get(&edge.to) else {
                continue;
            };

            used_text_nodes.insert(edge.from);
            used_text_nodes.insert(edge.to);

            let from = from.clone();
            let to = to.clone();

            let text = edge.label.as_ref().map(|text| Cow::Borrowed(&text[..]));
            let variant = edge.variant;

            let from_marker = (!edge.from.is_ascii_uppercase()).then_some(edge.from);
            let to_marker = (!edge.to.is_ascii_uppercase()).then_some(edge.to);

            lines.push(LineEdge {
                from,
                from_marker,
                to,
                to_marker,
                text,
                variant,
            });
        }

        self.text_nodes
            .retain(|n| !used_text_nodes.contains(&n.text()) && !n.text().is_ascii_uppercase());

        LineEdgeMarkers {
            lines,
            text_nodes: self.text_nodes,
        }
    }
}

impl LineEdgeMarkers<'_> {
    /// The edge lines for a [`LineEdgeMarkers`]
    pub fn lines(&self) -> &[LineEdge] {
        &self.lines
    }

    /// The lone standing text nodes for a [`LineEdgeMarkers`]
    pub fn text_nodes(&self) -> &[LineEdgeText] {
        &self.text_nodes
    }
}

impl LineEdge<'_> {
    /// The starting position
    #[inline]
    pub fn from(&self) -> &InSignalPosition {
        &self.from
    }

    /// The ending position
    #[inline]
    pub fn to(&self) -> &InSignalPosition {
        &self.to
    }

    /// The marker at the start of the edge line
    #[inline]
    pub fn from_marker(&self) -> Option<char> {
        self.from_marker
    }

    /// The marker at the end of the edge line
    #[inline]
    pub fn to_marker(&self) -> Option<char> {
        self.to_marker
    }

    /// The variant of the edge line
    #[inline]
    pub fn variant(&self) -> &EdgeVariant {
        &self.variant
    }

    /// The label text of the edge line
    #[inline]
    pub fn label(&self) -> Option<&str> {
        self.text.as_ref().map(|s| &s[..])
    }
}

impl LineEdgeText {
    /// The location of the node
    #[inline]
    pub fn at(&self) -> &InSignalPosition {
        &self.at
    }

    /// The text content of the node
    #[inline]
    pub fn text(&self) -> char {
        self.text
    }
}

impl InSignalPosition {
    /// The `x` value of the position
    pub fn x(&self) -> CycleOffset {
        self.x
    }

    /// The `y` value of the position
    pub fn y(&self) -> u32 {
        self.y
    }
}

impl EdgeDefinition {
    /// Create a new [`EdgeDefinition`] from a set of parameters
    pub fn new(variant: EdgeVariant, from: char, to: char, label: Option<String>) -> Self {
        Self { variant, from, to, label }
    }
}

impl FromStr for EdgeDefinition {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let start_len = s.len();
        let s = s.trim_start();
        let start_idx = start_len - s.len();

        let (s, from) = take(s).ok_or(start_idx)?;

        let start_len = s.len();
        let s = s.trim_start();
        let start_idx = start_idx + start_len - s.len();

        let (s, variant) = EdgeVariant::consume(s).ok_or(start_idx)?;

        let start_len = s.len();
        let s = s.trim_start();
        let start_idx = start_idx + start_len - s.len();

        let (s, to) = take(s).ok_or(start_idx)?;

        let text = (!s.is_empty()).then_some(s.trim_start().to_string());

        Ok(Self {
            variant,
            from,
            to,
            label: text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_variant_parse() {
        macro_rules! assert_edge_variant {
            ($input:literal, $output:expr) => {
                #[allow(unused)]
                use EdgeArrowType::*;
                #[allow(unused)]
                use EdgeVariant::*;

                let out = EdgeVariant::consume($input);

                assert!(out.is_some());

                let (out_str, out) = out.unwrap();

                assert!(out_str.is_empty());
                assert_eq!(out, $output);
            };
            ($input:literal) => {
                assert!(EdgeVariant::consume($input).is_none());
            };
        }

        assert_edge_variant!("-|-", Sharp(SharpEdgeVariant::BothHorizontal(None)));
        assert_edge_variant!("<-|-", Sharp(SharpEdgeVariant::BothHorizontal(Start)));
        assert_edge_variant!("<-|->", Sharp(SharpEdgeVariant::BothHorizontal(Both)));
        assert_edge_variant!("-|->", Sharp(SharpEdgeVariant::BothHorizontal(End)));
        assert_edge_variant!("->", Sharp(SharpEdgeVariant::Straight(End)));
        assert_edge_variant!("+", Sharp(SharpEdgeVariant::Cross));
        assert_edge_variant!("<+");
        assert_edge_variant!("<+>");
    }

    #[test]
    fn edge_definition() {
        macro_rules! assert_edge_def {
            ($input:literal => $from:literal, $to:literal, $edge_variant:expr, $text:expr) => {
                #[allow(unused)]
                use EdgeVariant::*;

                let out = EdgeDefinition::from_str($input);

                assert!(out.is_ok());

                let out = out.unwrap();

                assert_eq!(out.from, $from);
                assert_eq!(out.to, $to);
                assert_eq!(out.variant, $edge_variant);
                assert_eq!(out.label, Some($text).map(Into::into));
            };
            ($input:literal => $from:literal, $to:literal, $edge_variant:expr) => {
                #[allow(unused)]
                use EdgeVariant::*;

                let out = EdgeDefinition::from_str($input);

                assert!(out.is_ok());

                let out = out.unwrap();

                assert_eq!(out.from, $from);
                assert_eq!(out.to, $to);
                assert_eq!(out.variant, $edge_variant);
                assert!(out.label.is_none());
            };
            ($input:literal) => {
                assert!(EdgeDefinition::from_str($input).is_err());
            };
        }

        assert_edge_def!("I+J abc" => 'I', 'J', Sharp(SharpEdgeVariant::Cross), "abc");
        assert_edge_def!("I<+J abc");
        assert_edge_def!("<+J" => '<', 'J', Sharp(SharpEdgeVariant::Cross));
    }
}
