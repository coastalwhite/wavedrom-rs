use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

use crate::{CycleOffset, Signal};

#[derive(Debug, Clone)]
pub struct LineEdgeMarkers<'a> {
    lines: Vec<LineEdge<'a>>,
    text_nodes: Vec<LineEdgeText>,
}

#[derive(Debug, Clone)]
pub struct LineEdge<'a> {
    from: InSignalPosition,
    to: InSignalPosition,
    text: Option<Cow<'a, str>>,
    variant: EdgeVariant,
}

#[derive(Debug, Clone)]
pub struct LineEdgeText {
    at: InSignalPosition,
    text: char,
}

#[derive(Debug, Clone)]
pub struct InSignalPosition {
    x: CycleOffset,
    y: u32,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct EdgeDefinition {
    variant: EdgeVariant,
    from: char,
    to: char,
    text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeVariant {
    Spline(SplineEdgeVariant),
    Sharp(SharpEdgeVariant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplineEdgeVariant {
    BothHorizontal(EdgeArrowType),
    StartHorizontal(EdgeArrowType),
    EndHorizontal(EdgeArrowType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharpEdgeVariant {
    Straight(EdgeArrowType),
    BothHorizontal(EdgeArrowType),
    StartHorizontal(EdgeArrowType),
    EndHorizontal(EdgeArrowType),
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeArrowType {
    None,
    Start,
    End,
    Both,
}

pub(crate) struct LineEdgeMarkersBuilder {
    line_number: u32,
    node_positions: HashMap<char, InSignalPosition>,
    text_nodes: Vec<LineEdgeText>,
}

impl EdgeArrowType {
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

impl EdgeVariant {
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

        for (i, c) in signal.node.chars().enumerate() {
            if c == '.' {
                continue;
            }

            let at = InSignalPosition {
                x: signal.phase + CycleOffset::new_rounded(i as u32),
                y: line_number,
            };

            self.node_positions.insert(c, at.clone());

            if c.is_ascii_uppercase() {
                self.text_nodes.push(LineEdgeText { at, text: c });
            }
        }

        self.line_number += 1;
    }

    pub fn build<'a>(self, edges: &'a [EdgeDefinition]) -> LineEdgeMarkers<'a> {
        let mut lines = Vec::new();

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

            let from = from.clone();
            let to = to.clone();

            let text = if let Some(text) = &edge.text {
                Some(Cow::Borrowed(&text[..]))
            } else {
                None
            };
            let variant = edge.variant;

            lines.push(LineEdge {
                from,
                to,
                text,
                variant,
            });
        }

        LineEdgeMarkers {
            lines,
            text_nodes: self.text_nodes,
        }
    }
}

impl LineEdgeMarkers<'_> {
    pub fn lines(&self) -> &[LineEdge] {
        &self.lines
    }

    pub fn text_nodes(&self) -> &[LineEdgeText] {
        &self.text_nodes
    }
}

impl LineEdge<'_> {
    pub fn from(&self) -> &InSignalPosition {
        &self.from
    }

    pub fn to(&self) -> &InSignalPosition {
        &self.to
    }

    pub fn variant(&self) -> &EdgeVariant {
        &self.variant
    }
}

impl InSignalPosition {
    pub fn x(&self) -> CycleOffset {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
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
            text,
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
                assert_eq!(out.text, Some($text).map(Into::into));
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
                assert!(out.text.is_none());
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
