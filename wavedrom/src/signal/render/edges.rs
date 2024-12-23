use std::fmt::Display;
use std::io;

use super::dimensions::SvgDimensions;

use crate::escape::escape_str;
use crate::signal::edges::{
    EdgeArrowType, EdgeVariant, LineEdge, SharpEdgeVariant, SplineEdgeVariant,
};
use crate::{Color, Font, Options};

/// A f64 type that automatically rounds when formatting
struct SVGF64(pub f64);

impl Display for SVGF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:.3}", self.0);
        write!(f, "{}", s.trim_end_matches('0').trim_end_matches('.'))
    }
}

struct PlacedVec2D {
    origin: (f64, f64),
    dir: (f64, f64),
}

impl PlacedVec2D {
    fn invert(&self) -> Self {
        Self {
            origin: self.origin,
            dir: (-self.dir.0, -self.dir.1),
        }
    }
}

struct BBox {
    middle_x: u32,
    middle_y: u32,
    width: u32,
    height: u32,
}

pub fn write_line_edge(
    writer: &mut impl io::Write,
    edge: LineEdge,
    dims: &SvgDimensions,
    options: &Options,
    font: &Font,
) -> io::Result<(f64, f64)> {
    let path_options = options.signal.path;
    let edge_options = &options.signal.edge;

    let from = edge.from();
    let to = edge.to();

    let from_x = dims.schema_x() + from.x().width_offset(path_options.cycle_width.into());
    let from_y = dims.signal_top(from.y()) + u32::from(path_options.signal_height / 2);

    if from == to {
        return Ok((f64::from(from_x), f64::from(from_y)));
    }

    let to_x = dims.schema_x() + to.x().width_offset(path_options.cycle_width.into());
    let to_y = dims.signal_top(to.y()) + u32::from(path_options.signal_height / 2);

    let from_bbox = edge
        .from_marker()
        .map(|c| get_text_bbox(&c.to_string(), from_x, from_y, font, 14))
        .unwrap_or(BBox::at(from_x, from_y));
    let to_bbox = edge
        .to_marker()
        .map(|c| get_text_bbox(&c.to_string(), to_x, to_y, font, 14))
        .unwrap_or(BBox::at(to_x, to_y));

    let (start, end) = if from_x == to_x {
        if from_y < to_y {
            (
                PlacedVec2D::down(from_x, from_bbox.y_max()),
                PlacedVec2D::down(to_x, to_bbox.y_min()),
            )
        } else {
            (
                PlacedVec2D::up(from_x, from_bbox.y_min()),
                PlacedVec2D::up(to_x, to_bbox.y_max()),
            )
        }
    } else if from_y == to_y {
        if from_x < to_x {
            (
                PlacedVec2D::right(from_bbox.x_max(), from_y),
                PlacedVec2D::right(to_bbox.x_min(), to_y),
            )
        } else {
            (
                PlacedVec2D::left(from_bbox.x_min(), from_y),
                PlacedVec2D::left(to_bbox.x_max(), to_y),
            )
        }
    } else {
        match edge.variant() {
            EdgeVariant::Spline(spline_edge) => match spline_edge {
                SplineEdgeVariant::BothHorizontal(_) => {
                    if from_x < to_x {
                        (
                            PlacedVec2D::right(from_bbox.x_max(), from_y),
                            PlacedVec2D::right(to_bbox.x_min(), to_y),
                        )
                    } else {
                        (
                            PlacedVec2D::left(from_bbox.x_min(), from_y),
                            PlacedVec2D::left(to_bbox.x_max(), to_y),
                        )
                    }
                }
                SplineEdgeVariant::StartHorizontal(_) => {
                    const C1_FACTOR: f64 = 0.25;
                    const C2_FACTOR: f64 = 0.8;

                    let dx = f64::from(to_x) - f64::from(from_x);

                    let cx1 = f64::from(from_x) + dx * C1_FACTOR;
                    let cy1 = f64::from(from_y);
                    let cx2 = f64::from(from_x) + dx * C2_FACTOR;
                    let cy2 = f64::from(from_y);

                    let start_dir = (cx1 - f64::from(from_x), 0.);
                    let end_dir = (f64::from(to_x) - cx2, f64::from(to_y) - cy2);

                    (
                        PlacedVec2D {
                            origin: from_bbox.intersection_bb(cx1, cy1),
                            dir: start_dir,
                        },
                        PlacedVec2D {
                            origin: to_bbox.intersection_bb(cx2, cy2),
                            dir: end_dir,
                        },
                    )
                }
                SplineEdgeVariant::EndHorizontal(_) => {
                    const C1_FACTOR: f64 = 0.2;
                    const C2_FACTOR: f64 = 0.75;

                    let dx = f64::from(to_x) - f64::from(from_x);

                    let cx1 = f64::from(from_x) + dx * C1_FACTOR;
                    let cy1 = f64::from(to_y);
                    let cx2 = f64::from(from_x) + dx * C2_FACTOR;
                    let cy2 = f64::from(to_y);

                    let start_dir = (cx1 - f64::from(from_x), cy1 - f64::from(from_y));
                    let end_dir = (f64::from(to_x) - cx2, 0.);

                    (
                        PlacedVec2D {
                            origin: from_bbox.intersection_bb(cx1, cy1),
                            dir: start_dir,
                        },
                        PlacedVec2D {
                            origin: to_bbox.intersection_bb(cx2, cy2),
                            dir: end_dir,
                        },
                    )
                }
            },
            EdgeVariant::Sharp(sharp_edge) => match sharp_edge {
                SharpEdgeVariant::Straight(_) | SharpEdgeVariant::Cross => {
                    let dir = (
                        f64::from(to_x) - f64::from(from_x),
                        f64::from(to_y) - f64::from(from_y),
                    );

                    let from_intersect = from_bbox.intersection_bb(to_x, to_y);
                    let to_intersect = to_bbox.intersection_bb(from_x, from_y);

                    (
                        PlacedVec2D {
                            origin: from_intersect,
                            dir,
                        },
                        PlacedVec2D {
                            origin: to_intersect,
                            dir,
                        },
                    )
                }
                SharpEdgeVariant::BothHorizontal(_) => {
                    if from_x < to_x {
                        (
                            PlacedVec2D::right(from_bbox.x_max(), from_y),
                            PlacedVec2D::right(to_bbox.x_min(), to_y),
                        )
                    } else {
                        (
                            PlacedVec2D::left(from_bbox.x_min(), from_y),
                            PlacedVec2D::left(to_bbox.x_max(), to_y),
                        )
                    }
                }
                SharpEdgeVariant::StartHorizontal(_) => {
                    if from_x < to_x {
                        (
                            PlacedVec2D::right(from_bbox.x_max(), from_y),
                            if from_y < to_y {
                                PlacedVec2D::down(to_x, to_bbox.y_min())
                            } else {
                                PlacedVec2D::up(to_x, to_bbox.y_max())
                            },
                        )
                    } else {
                        (
                            PlacedVec2D::left(from_bbox.x_min(), from_y),
                            if from_y < to_y {
                                PlacedVec2D::down(to_x, to_bbox.y_min())
                            } else {
                                PlacedVec2D::up(to_x, to_bbox.y_max())
                            },
                        )
                    }
                }
                SharpEdgeVariant::EndHorizontal(_) => {
                    if from_x < to_x {
                        (
                            if from_y < to_y {
                                PlacedVec2D::down(from_x, from_bbox.y_max())
                            } else {
                                PlacedVec2D::up(from_x, from_bbox.y_min())
                            },
                            PlacedVec2D::right(to_bbox.x_min(), to_y),
                        )
                    } else {
                        (
                            if from_y < to_y {
                                PlacedVec2D::down(from_x, from_bbox.y_max())
                            } else {
                                PlacedVec2D::up(from_x, from_bbox.y_min())
                            },
                            PlacedVec2D::left(to_bbox.x_max(), to_y),
                        )
                    }
                }
            },
        }
    };

    let arrow_type = edge.variant().arrow_type();
    let offset_start = if arrow_type.has_start_arrow() {
        offset_in_dir(start.origin, start.dir, 4.).unwrap_or(start.origin)
    } else if matches!(edge.variant(), EdgeVariant::Sharp(SharpEdgeVariant::Cross)) {
        offset_in_dir(start.origin, start.dir, 2.).unwrap_or(start.origin)
    } else {
        start.origin
    };

    let offset_end = if arrow_type.has_end_arrow() {
        offset_in_dir(end.origin, end.dir, -4.).unwrap_or(end.origin)
    } else if matches!(edge.variant(), EdgeVariant::Sharp(SharpEdgeVariant::Cross)) {
        offset_in_dir(end.origin, end.dir, -2.).unwrap_or(end.origin)
    } else {
        end.origin
    };

    let (start_x, start_y) = (SVGF64(offset_start.0), SVGF64(offset_start.1));
    let (end_x, end_y) = (SVGF64(offset_end.0), SVGF64(offset_end.1));

    write!(writer, r##"<g><path d="M{start_x},{start_y}"##)?;

    let (middle_x, middle_y) = if to_x == from_x {
        write!(writer, "V{end_y}")?;
        (
            f64::from(from_x),
            (f64::from(from_y) + f64::from(to_y)) / 2.,
        )
    } else if to_y == from_y {
        write!(writer, "H{end_x}")?;
        (
            (f64::from(from_x) + f64::from(to_x)) / 2.,
            f64::from(from_y),
        )
    } else {
        match *edge.variant() {
            EdgeVariant::Spline(spline_edge) => match spline_edge {
                SplineEdgeVariant::BothHorizontal(_) => {
                    write!(
                        writer,
                        "C{hx},{from_y} {hx},{to_y} {end_x},{end_y}",
                        hx = (from_x + to_x) / 2,
                    )?;

                    (
                        (f64::from(from_x) + f64::from(to_x)) / 2.,
                        (f64::from(from_y) + f64::from(to_y)) / 2.,
                    )
                }
                SplineEdgeVariant::StartHorizontal(_) => {
                    let cx1 = start.origin.0 + start.dir.0;
                    let cy1 = start.origin.1 + start.dir.1;
                    let cx2 = end.origin.0 - end.dir.0;
                    let cy2 = end.origin.1 - end.dir.1;

                    write!(writer, "C{cx1},{cy1} {cx2},{cy2} {end_x},{end_y}",)?;

                    (
                        ((start.origin.0 + end.origin.0) / 2. + cx1 + cx2) / 3.,
                        ((start.origin.1 + end.origin.1) / 2. + cy1 + cy2) / 3.,
                    )
                }
                SplineEdgeVariant::EndHorizontal(_) => {
                    let cx1 = start.origin.0 + start.dir.0;
                    let cy1 = start.origin.1 + start.dir.1;
                    let cx2 = end.origin.0 - end.dir.0;
                    let cy2 = end.origin.1 - end.dir.1;

                    write!(writer, "C{cx1},{cy1} {cx2},{cy2} {end_x},{end_y}",)?;

                    (
                        ((start.origin.0 + end.origin.0) / 2. + cx1 + cx2) / 3.,
                        ((start.origin.1 + end.origin.1) / 2. + cy1 + cy2) / 3.,
                    )
                }
            },
            EdgeVariant::Sharp(sharp_edge) => match sharp_edge {
                SharpEdgeVariant::Straight(_) | SharpEdgeVariant::Cross => {
                    write!(writer, "L{end_x},{end_y}")?;

                    (
                        (f64::from(from_x) + f64::from(to_x)) / 2.,
                        (f64::from(from_y) + f64::from(to_y)) / 2.,
                    )
                }
                SharpEdgeVariant::BothHorizontal(_) => {
                    write!(writer, "H{hx}V{end_y}H{end_x}", hx = (from_x + to_x) / 2)?;

                    (
                        (f64::from(from_x) + f64::from(to_x)) / 2.,
                        (f64::from(from_y) + f64::from(to_y)) / 2.,
                    )
                }
                SharpEdgeVariant::StartHorizontal(_) => {
                    write!(writer, "H{end_x}V{end_y}")?;

                    (f64::from(to_x), f64::from(from_y))
                }
                SharpEdgeVariant::EndHorizontal(_) => {
                    write!(writer, "V{end_y}H{end_x}")?;

                    (f64::from(from_x), f64::from(to_y))
                }
            },
        }
    };

    if matches!(edge.variant(), EdgeVariant::Sharp(SharpEdgeVariant::Cross)) {
        const MHEIGHT: u32 = 5;

        if to_x == from_y {
            let top_x = offset_start.0 - f64::from(MHEIGHT);
            write!(
                writer,
                "M{top_x},{start_y}h{height}M{top_x},{end_y}h{height}",
                height = 2 * MHEIGHT
            )?;
        } else if to_y == from_y {
            let top_y = offset_start.1 - f64::from(MHEIGHT);
            write!(
                writer,
                "M{start_x},{top_y}v{height}M{end_x},{top_y}v{height}",
                height = 2 * MHEIGHT
            )?;
        } else if let Some((xoffset, yoffset)) = offset_in_dir(
            (0, 0),
            (
                f64::from(from_y) - f64::from(to_y),
                f64::from(to_x) - f64::from(from_x),
            ),
            MHEIGHT,
        ) {
            write!(
                writer,
                "M{x1},{y1}L{x2},{y2}",
                x1 = offset_start.0 + xoffset,
                y1 = offset_start.1 + yoffset,
                x2 = offset_start.0 - xoffset,
                y2 = offset_start.1 - yoffset,
            )?;

            write!(
                writer,
                "M{x1},{y1}L{x2},{y2}",
                x1 = offset_end.0 + xoffset,
                y1 = offset_end.1 + yoffset,
                x2 = offset_end.0 - xoffset,
                y2 = offset_end.1 - yoffset,
            )?;
        }
    }

    write!(
        writer,
        r##"" fill="none" stroke="{color}" stroke-width="1"/>"##,
        color = edge_options.edge_color,
    )?;

    write_edge_arrow_heads(
        writer,
        arrow_type,
        start,
        end,
        edge_options.edge_arrow_size,
        edge_options.edge_arrow_color,
    )?;

    write!(writer, "</g>")?;

    Ok((middle_x, middle_y))
}

pub fn write_line_edge_markers(
    writer: &mut impl io::Write,
    edge: LineEdge,
    middle: (f64, f64),
    dims: &SvgDimensions,
    options: &Options,
    font: &Font,
) -> io::Result<()> {
    let path_options = options.signal.path;
    let edge_options = &options.signal.edge;

    let from = edge.from();
    let to = edge.to();

    if from == to {
        return Ok(());
    }

    let from_x = dims.schema_x() + from.x().width_offset(path_options.cycle_width.into());
    let from_y = dims.signal_top(from.y()) + u32::from(path_options.signal_height / 2);

    let to_x = dims.schema_x() + to.x().width_offset(path_options.cycle_width.into());
    let to_y = dims.signal_top(to.y()) + u32::from(path_options.signal_height / 2);

    let middle_x = middle.0;
    let middle_y = middle.1;

    if let Some(c) = edge.from_marker() {
        write_edge_text(
            writer,
            (f64::from(from_x), f64::from(from_y)),
            &c.to_string(),
            edge_options.node_font_size,
            edge_options.node_text_color,
            edge_options.node_background_color,
            font,
        )?;
    }

    if let Some(c) = edge.to_marker() {
        write_edge_text(
            writer,
            (f64::from(to_x), f64::from(to_y)),
            &c.to_string(),
            edge_options.node_font_size,
            edge_options.node_text_color,
            edge_options.node_background_color,
            font,
        )?;
    }

    if let Some(text) = edge.label() {
        write_edge_text(
            writer,
            (middle_x, middle_y),
            text,
            edge_options.edge_text_font_size,
            edge_options.edge_text_color,
            edge_options.edge_text_background_color,
            font,
        )?;
    }

    Ok(())
}

impl BBox {
    fn at(x: u32, y: u32) -> Self {
        Self {
            middle_x: x,
            middle_y: y,
            width: 0,
            height: 0,
        }
    }

    fn x_min(&self) -> f64 {
        f64::from(self.middle_x) - f64::from(self.width) / 2.
    }
    fn x_max(&self) -> f64 {
        f64::from(self.middle_x) + f64::from(self.width) / 2.
    }

    fn y_min(&self) -> f64 {
        f64::from(self.middle_y) - f64::from(self.height) / 2.
    }
    fn y_max(&self) -> f64 {
        f64::from(self.middle_y) + f64::from(self.height) / 2.
    }

    fn intersection_bb(&self, to_x: impl Into<f64>, to_y: impl Into<f64>) -> (f64, f64) {
        if self.width == 0 || self.height == 0 {
            return (self.middle_x.into(), self.middle_y.into());
        }

        let dir = (
            to_x.into() - f64::from(self.middle_x),
            to_y.into() - f64::from(self.middle_y),
        );

        let bbox_x = dir.0.signum() * f64::from(self.width) / 2.;
        let bbox_y = dir.1.signum() * f64::from(self.height) / 2.;

        let bbox_x_intersection_y = bbox_x * (dir.1 / dir.0);
        let bbox_y_intersection_x = bbox_y * (dir.0 / dir.1);

        let bbox_x_dis = bbox_x * bbox_x + bbox_x_intersection_y * bbox_x_intersection_y;
        let bbox_y_dis = bbox_y * bbox_y + bbox_y_intersection_x * bbox_y_intersection_x;

        if bbox_x_dis < bbox_y_dis {
            (
                f64::from(self.middle_x) + bbox_x,
                f64::from(self.middle_y) + bbox_x_intersection_y,
            )
        } else {
            (
                f64::from(self.middle_x) + bbox_y_intersection_x,
                f64::from(self.middle_y) + bbox_y,
            )
        }
    }
}

impl PlacedVec2D {
    #[inline]
    fn up(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            origin: (x.into(), y.into()),
            dir: (0., -1.),
        }
    }

    #[inline]
    fn right(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            origin: (x.into(), y.into()),
            dir: (1., 0.),
        }
    }

    #[inline]
    fn down(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            origin: (x.into(), y.into()),
            dir: (0., 1.),
        }
    }

    #[inline]
    fn left(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            origin: (x.into(), y.into()),
            dir: (-1., 0.),
        }
    }
}

fn get_text_bbox(text: &str, middle_x: u32, middle_y: u32, font: &Font, font_size: u32) -> BBox {
    let width = font.get_text_width(text, 14);

    BBox {
        middle_x,
        middle_y,
        width,
        height: font_size,
    }
}

pub fn write_edge_text(
    writer: &mut impl io::Write,
    at: (f64, f64),
    text: &str,
    font_size: u32,
    text_color: Color,
    background_color: Color,
    font: &Font,
) -> io::Result<()> {
    let width = font.get_text_width(text, font_size);
    let font_family = font
        .get_font_family_name()
        .unwrap_or_else(|| "Helvetica".to_string());

    let rect_x = at.0 - f64::from(width) / 2.;
    let rect_y = at.1 - f64::from(font_size) / 2.;

    write!(
        writer,
        r##"<g><rect x="{rect_x}" y="{rect_y}" width="{width}" height="{font_size}" stroke="none" fill="{background_color}"/><text x="{text_x}" y="{text_y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0" fill="{text_color}"><tspan>{text}</tspan></text></g>"##,
        text_x = at.0,
        text_y = at.1,
        text = escape_str(text),
    )?;

    Ok(())
}

fn write_edge_arrow_head_path(
    writer: &mut impl io::Write,
    v2d: PlacedVec2D,
    size: u32,
) -> io::Result<()> {
    let Some(end) = offset_in_dir(v2d.origin, v2d.dir, size) else {
        return Ok(());
    };

    let Some(v1) = offset_in_dir(end, (-v2d.dir.1, v2d.dir.0), f64::from(size / 2)) else {
        return Ok(());
    };

    let Some(v2) = offset_in_dir(end, (-v2d.dir.1, v2d.dir.0), -f64::from(size / 2)) else {
        return Ok(());
    };

    let at_x = SVGF64(v2d.origin.0);
    let at_y = SVGF64(v2d.origin.1);

    let v1x = SVGF64(v1.0);
    let v1y = SVGF64(v1.1);

    let v2x = SVGF64(v2.0);
    let v2y = SVGF64(v2.1);

    write!(writer, "M{at_x},{at_y}L{v1x},{v1y}L{v2x},{v2y}z")
}

fn write_edge_arrow_heads(
    writer: &mut impl io::Write,
    arrow_type: EdgeArrowType,
    begin: PlacedVec2D,
    end: PlacedVec2D,
    arrow_size: u32,
    arrow_color: Color,
) -> io::Result<()> {
    write!(writer, r#"<path d=""#)?;

    if matches!(arrow_type, EdgeArrowType::Start | EdgeArrowType::Both) {
        write_edge_arrow_head_path(writer, begin, arrow_size)?;
    }

    if matches!(arrow_type, EdgeArrowType::End | EdgeArrowType::Both) {
        write_edge_arrow_head_path(writer, end.invert(), arrow_size)?;
    }

    write!(writer, r##"" fill="{arrow_color}" stroke="none"/>"##)?;

    Ok(())
}

fn offset_in_dir(
    p: (impl Into<f64>, impl Into<f64>),
    dir: (impl Into<f64>, impl Into<f64>),
    amount: impl Into<f64>,
) -> Option<(f64, f64)> {
    let p = (p.0.into(), p.1.into());
    let dir = (dir.0.into(), dir.1.into());
    let amount = amount.into();

    if dir == (0., 0.) {
        None
    } else if dir.0 == 0. {
        Some((p.0, p.1 + dir.1.signum() * amount))
    } else if dir.1 == 0. {
        Some((p.0 + dir.0.signum() * amount, p.1))
    } else {
        let dydx = dir.1 / dir.0;

        let xoffset = dir.0.signum() * amount / (1. + dydx * dydx).sqrt();
        let yoffset = dydx * xoffset;

        Some((p.0 + xoffset, p.1 + yoffset))
    }
}
