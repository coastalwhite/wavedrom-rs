use std::borrow::Cow;
use std::io;

use crate::path::{PathCommand, PathSegmentBackground};
use crate::{ClockEdge, SignalOptions};

use super::path::AssembledSignalPath;
use super::AssembledFigure;

mod font;
mod dimensions;
pub mod options;

pub use font::Font;
use options::RenderOptions;
use dimensions::SvgDimensions;

pub trait ToSvg {
    type Options: Default;

    fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Self::Options,
    ) -> io::Result<()>;

    #[inline]
    fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        self.write_svg_with_options(writer, &Self::Options::default())
    }
}

fn escape_str(s: &str) -> Cow<str> {
    if !s.contains(&['<', '>', '"', '&']) {
        return Cow::Borrowed(s);
    }

    let mut output = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            _ => output.push(c),
        }
    }
    Cow::Owned(output)
}

fn gap(writer: &mut impl io::Write, wave_height: u16) -> io::Result<()> {
    let wave_height = f32::from(wave_height);

    let a: f32 = 8.0;
    let b = wave_height / 2.0 + 6.0;

    const DISTANCE: f32 = 4.0;

    let start = (-a, b);
    let end = (a, -b);

    let control_1 = (-a / 2.0, b);

    let rad = (-2.0 * b / a).atan();
    let control_2 = (rad.cos() * a / -2.0, rad.sin() * a / -2.0);

    let control_3 = (a / 2.0, -b);

    write!(
        writer,
        r##"<path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}H{rp1x}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}H{lp1x}z" fill="#fff" stroke="none"/><path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}" fill="none" stroke="#000" stroke-width="1"/><path d="M{rp1x},{rp1y}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}" fill="none" stroke="#000" stroke-width="1"/>"##,
        lp1x = start.0 - DISTANCE / 2.0,
        lp1y = start.1,
        lp2x = control_1.0 - DISTANCE / 2.0,
        lp2y = control_1.1,
        lp3x = control_2.0 - DISTANCE / 2.0,
        lp3y = control_2.1,
        lp4x = 0.0 - DISTANCE / 2.0,
        lp4y = 0,
        lp5x = control_3.0 - DISTANCE / 2.0,
        lp5y = control_3.1,
        lp6x = end.0 - DISTANCE / 2.0,
        lp6y = end.1,
        rp1x = end.0 + DISTANCE / 2.0,
        rp1y = end.1,
        rp2x = control_3.0 + DISTANCE / 2.0,
        rp2y = control_3.1,
        rp3x = -control_2.0 + DISTANCE / 2.0,
        rp3y = -control_2.1,
        rp4x = 0.0 + DISTANCE / 2.0,
        rp4y = 0,
        rp5x = control_1.0 + DISTANCE / 2.0,
        rp5y = control_1.1,
        rp6x = start.0 + DISTANCE / 2.0,
        rp6y = start.1,
    )
}

fn posedge_arrow(writer: &mut impl io::Write, wave_height: u16) -> io::Result<()> {
    let scale = i32::from(wave_height / 6);

    write!(
        writer,
        r##"<path d="M{x1},{y1}L{x2},{y2}L{x3},{y3}H{hback}z" fill="#000" stroke="none"/>"##,
        x1 = -scale,
        y1 = scale,
        x2 = 0,
        y2 = -scale,
        x3 = scale,
        y3 = scale,
        hback = -scale * 2,
    )
}

fn negedge_arrow(writer: &mut impl io::Write, wave_height: u16) -> io::Result<()> {
    let scale = i32::from(wave_height / 6);

    write!(
        writer,
        r##"<path d="M{x1},{y1}L{x2},{y2}L{x3},{y3}H{hback}z" fill="#000" stroke="none"/>"##,
        x1 = -scale,
        y1 = -scale,
        x2 = 0,
        y2 = scale,
        x3 = scale,
        y3 = -scale,
        hback = -scale * 2,
    )
}

impl<'a> ToSvg for AssembledFigure<'a> {
    type Options = RenderOptions;

    fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Self::Options,
    ) -> io::Result<()> {
        let RenderOptions {
            font_size,
            background,
            paddings,
            spacings,
            wave_dimensions,
            group_indicator_dimensions,
            header,
            footer,
        } = options;

        let font = Font::default();
        let font_family = font
            .get_font_family_name()
            .unwrap_or_else(|| "helvetica".to_string());

        let dims = SvgDimensions::new(self, font, options);

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}" overflow="hidden" width="{figure_width}" height="{figure_height}">"#,
            figure_width = dims.figure_width(),
            figure_height = dims.figure_height(),
        )?;

        write!(writer, "<defs>")?;
        if self.definitions.has_undefined {
            write!(
                writer,
                r##"<pattern id="x-bg" patternUnits="userSpaceOnUse" width="4" height="10" patternTransform="rotate(45)"><line x1="0" y="0" x2="0" y2="10" stroke="#000" stroke-width="1"/></pattern>"##
            )?;
        }

        if self.definitions.has_posedge_marker {
            write!(writer, r##"<g id="pei">"##)?;
            posedge_arrow(writer, wave_dimensions.signal_height)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_negedge_marker {
            write!(writer, r##"<g id="nei">"##)?;
            negedge_arrow(writer, wave_dimensions.signal_height)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_gap {
            write!(writer, r##"<g id="gap">"##)?;
            gap(writer, wave_dimensions.signal_height)?;
            write!(writer, r##"</g>"##)?;
        }

        // Define the cycle-to-cycle background hint lines
        write!(
            writer,
            r##"<g id="cl"><path fill="none" d="M0,0v{schema_height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC"/></g>"##,
            schema_height = dims.schema_height(),
        )?;
        write!(writer, "</defs>")?;

        if let Some(background) = background {
            write!(
                writer,
                r##"<rect width="100%" height="100%" fill="{background}"/>"##
            )?;
        }

        // Figure container
        write!(
            writer,
            r##"<g transform="translate({padding_x},{padding_y})">"##,
            padding_x = paddings.figure_left,
            padding_y = paddings.figure_top,
        )?;

        // --- Start Header ---
        if let Some(title) = self.title {
            let title_font_size = header.font_size;
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{title_font_size}" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = dims.header_width() / 2,
                y = dims.header_height() / 2,
                text = escape_str(title),
            )?;
        }
        if let Some(cycle_marker) = self.top_cycle_marker {
            let start = cycle_marker.start();
            let every = cycle_marker.every();

            let marker_font_size = header.cycle_marker_font_size;
            let end = start + self.num_cycles;

            if every != 0 {
                for offset in (start..end).step_by(every as usize) {
                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{marker_font_size}" letter-spacing="0"><tspan>{offset}</tspan></text>"##,
                        x = dims.schema_x()
                            + dims.cycle_width() * (offset - start)
                            + dims.cycle_width() / 2,
                        y = dims.header_height(),
                    )?;
                }
            }
        }
        // --- End Header ---

        write!(
            writer,
            r##"<g transform="translate({schema_x},{schema_y})">"##,
            schema_x = dims.schema_x(),
            schema_y = dims.schema_y(),
        )?;
        for i in 0..=self.num_cycles {
            write!(
                writer,
                r##"<use transform="translate({x})" xlink:href="#cl"/>"##,
                x = i * dims.cycle_width()
            )?;
        }
        write!(writer, r##"</g>"##)?;

        for group in self.groups.iter() {
            if group.is_empty() {
                continue;
            }

            let height = group.len() * u32::from(wave_dimensions.signal_height)
                + (group.len() - 1) * spacings.line_to_line;
            let x = self.amount_labels_before(group.depth() + 1)
                * group_indicator_dimensions.label_height()
                + if group.depth() == 0 {
                    0
                } else {
                    group.depth() * group_indicator_dimensions.width
                };
            let y = dims.header_height()
                + paddings.schema_top
                + if group.start() == 0 {
                    0
                } else {
                    group.start() * u32::from(wave_dimensions.signal_height)
                        + group.start() * spacings.line_to_line
                };

            if let Some(label) = group.label() {
                let x = if group.depth() == 0 {
                    0
                } else {
                    self.amount_labels_before(group.depth())
                        * group_indicator_dimensions.label_height()
                        + group.depth() * group_indicator_dimensions.width
                };

                // let label_width = get_text_width(label, &face, 8);
                write!(
                    writer,
                    r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0" transform="rotate(270)"><tspan>{text}</tspan></text></g>"##,
                    font_size = group_indicator_dimensions.label_fontsize,
                    x = x + group_indicator_dimensions.label_spacing + 2,
                    y = y + height / 2,
                    text = escape_str(label),
                )?;
            }

            write!(
                writer,
                r##"<path fill="none" d="M{x},{y}m{w},0c-3,0 -{w},1 -{w},{w}v{h}c0,3 1,{w} {w},{w}" stroke="#000"/>"##,
                h = height - group_indicator_dimensions.width * 2,
                w = group_indicator_dimensions.width,
            )?;
        }

        for (i, line) in self.lines.iter().enumerate() {
            if line.is_empty() {}

            let Ok(i) = u32::try_from(i) else {
                break;
            };

            let x = dims
                .has_textbox()
                .then_some(dims.textbox_x())
                .unwrap_or(dims.schema_x());
            let y = dims.header_height()
                + paddings.schema_top
                + if i == 0 {
                    0
                } else {
                    (u32::from(wave_dimensions.signal_height) + spacings.line_to_line) * i
                };

            write!(writer, r##"<g transform="translate({x},{y})">"##)?;

            if !line.text.is_empty() {
                write!(
                    writer,
                    r##"<g transform="translate(0,{y})"><text dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0"><tspan>{text}</tspan></text></g>"##,
                    font_size = font_size,
                    y = wave_dimensions.signal_height / 2,
                    text = escape_str(line.text),
                )?;
            }

            if dims.has_textbox() {
                write!(
                    writer,
                    r##"<g transform="translate({schema_x})">"##,
                    schema_x = dims.schema_x() - dims.textbox_x()
                )?;
                write_signal(&line.path, writer, &wave_dimensions, self.hscale)?;
                write!(writer, r##"</g>"##)?;
            } else {
                write_signal(&line.path, writer, &wave_dimensions, self.hscale)?;
            }

            write!(writer, r##"</g>"##)?;
        }

        // --- Start Footer ---
        if let Some(footer_text) = self.footer {
            let footer_font_size = footer.font_size;
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{footer_font_size}" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = dims.footer_width() / 2,
                y = dims.footer_y() + dims.footer_height() / 2,
                text = escape_str(footer_text),
            )?;
        }
        if let Some(cycle_marker) = self.bottom_cycle_marker {
            let start = cycle_marker.start();
            let every = cycle_marker.every();

            let marker_font_size = footer.cycle_marker_font_size;
            let end = start + self.num_cycles;

            if every != 0 {
                for offset in (start..end).step_by(every as usize) {
                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{marker_font_size}" letter-spacing="0"><tspan>{offset}</tspan></text>"##,
                        x = dims.schema_x()
                            + dims.cycle_width() * (offset - start)
                            + dims.cycle_width() / 2,
                        y = dims.footer_y()
                    )?;
                }
            }
        }
        // --- End Footer ---

        write!(writer, "</g></svg>")?;

        Ok(())
    }
}

fn draw_dashed_horizontal_line(writer: &mut impl io::Write, dx: i32) -> io::Result<()> {
    let mut cx = 0i32;

    loop {
        if cx.abs() == dx.abs() {
            break;
        }

        write!(writer, "h{signed_len}", signed_len = dx.signum() * 4)?;
        cx = i32::min(dx.abs(), cx + 4);

        if cx.abs() >= dx.abs() {
            break;
        }

        write!(writer, "m{signed_len},0", signed_len = dx.signum() * 4)?;
        cx = i32::min(dx.abs(), cx + 4);
    }

    Ok(())
}

fn write_signal(
    wave_path: &AssembledSignalPath,
    writer: &mut impl io::Write,
    options: &SignalOptions,
    hscale: u16,
) -> io::Result<()> {
    for segment in wave_path.segments() {
        let fill = match segment.background() {
            Some(PathSegmentBackground::B2) => &options.backgrounds[0],
            Some(PathSegmentBackground::B3) => &options.backgrounds[1],
            Some(PathSegmentBackground::B4) => &options.backgrounds[2],
            Some(PathSegmentBackground::B5) => &options.backgrounds[3],
            Some(PathSegmentBackground::B6) => &options.backgrounds[4],
            Some(PathSegmentBackground::B7) => &options.backgrounds[5],
            Some(PathSegmentBackground::B8) => &options.backgrounds[6],
            Some(PathSegmentBackground::B9) => &options.backgrounds[7],
            Some(PathSegmentBackground::Undefined) => "url(#x-bg)",
            None => "none",
        };

        let x = segment.x();
        let y = segment.y();

        write!(writer, r##"<path fill="{fill}" d=""##)?;
        write!(writer, "M{x},{y}")?;
        for action in segment.actions() {
            match action {
                PathCommand::LineVerticalNoStroke(dy) => write!(writer, "v{dy}"),
                PathCommand::LineVertical(dy) => write!(writer, "v{dy}"),
                PathCommand::LineHorizontal(dx) => write!(writer, "h{dx}"),
                PathCommand::DashedLineHorizontal(dx) => draw_dashed_horizontal_line(writer, *dx),
                PathCommand::Line(dx, dy) => write!(writer, "l{dx},{dy}"),
                PathCommand::Curve(cdx1, cdy1, cdx2, cdy2, dx, dy) => {
                    write!(writer, "c{cdx1},{cdy1} {cdx2},{cdy2} {dx},{dy}")
                }
            }?
        }

        if segment.background().is_some() {
            write!(writer, r##"z"##)?;
        }

        // If there is a `no_stroke` element, we need to divide up the filling and the
        // stroking.
        if !segment.is_fully_stroked() {
            write!(writer, r##"" stroke="none"/>"##)?;

            write!(writer, r##"<path fill="none" d=""##)?;
            write!(writer, "M{x},{y}")?;
            for action in segment.actions() {
                match action {
                    PathCommand::LineVerticalNoStroke(dy) => write!(writer, "m0,{dy}"),
                    PathCommand::LineVertical(dy) => write!(writer, "v{dy}"),
                    PathCommand::LineHorizontal(dx) => write!(writer, "h{dx}"),
                    PathCommand::DashedLineHorizontal(dx) => {
                        draw_dashed_horizontal_line(writer, *dx)
                    }
                    PathCommand::Line(dx, dy) => write!(writer, "l{dx},{dy}"),
                    PathCommand::Curve(cdx1, cdy1, cdx2, cdy2, dx, dy) => {
                        write!(writer, "c{cdx1},{cdy1} {cdx2},{cdy2} {dx},{dy}")
                    }
                }?
            }
        }
        write!(writer, r##"" stroke-width="1" stroke="#000"/>"##)?;

        if let Some(marker_text) = segment.marker_text() {
            write!(
                writer,
                r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0"><tspan>{text}</tspan></text></g>"##,
                font_family = "Helvetica",
                font_size = options.marker_font_size,
                text = marker_text,
                x = segment.x() + segment.width() / 2,
                y = options.signal_height / 2,
            )?;
        }

        for clock_edge_marker in segment.clock_edge_markers() {
            let x = clock_edge_marker
                .at()
                .width_offset(u32::from(options.cycle_width * hscale));
            let y = u32::from(options.signal_height) / 2;

            match clock_edge_marker.edge() {
                ClockEdge::Positive => {
                    write!(
                        writer,
                        r##"<use transform="translate({x},{y})" xlink:href="#pei"/>"##,
                    )?;
                }
                ClockEdge::Negative => {
                    write!(
                        writer,
                        r##"<use transform="translate({x},{y})" xlink:href="#nei"/>"##,
                    )?;
                }
            };
        }

        for gap in segment.gaps() {
            let x = gap.width_offset(u32::from(options.cycle_width * hscale));
            let y = u32::from(options.signal_height) / 2;

            write!(
                writer,
                r##"<use transform="translate({x},{y})" xlink:href="#gap"/>"##,
            )?;
        }
    }

    Ok(())
}
