use std::io;

use super::markers::ClockEdge;
use super::path::{PathCommand, PathSegmentBackground};
use crate::escape::escape_str;
use crate::{Color, Font, Options};

use self::edges::{write_edge_text, write_line_edge, write_line_edge_markers};

use super::path::AssembledSignalPath;
use super::AssembledFigure;

mod dimensions;
mod edges;

use super::options::{PathOptions, SignalOptions};
use dimensions::SvgDimensions;

fn gap(
    writer: &mut impl io::Write,
    wave_height: u16,
    color: Color,
    background: Color,
) -> io::Result<()> {
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
        r##"<path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}H{rp1x}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}H{lp1x}z" fill="{background}" stroke="none"/><path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}" fill="none" stroke="{color}" stroke-width="1"/><path d="M{rp1x},{rp1y}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}" fill="none" stroke="{color}" stroke-width="1"/>"##,
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

fn posedge_arrow(writer: &mut impl io::Write, wave_height: u32, color: Color) -> io::Result<()> {
    let scale = i64::from(wave_height / 6);

    write!(
        writer,
        r##"<path d="M{x1},{y1}L{x2},{y2}L{x3},{y3}H{hback}z" fill="{color}" stroke="none"/>"##,
        x1 = -scale,
        y1 = scale,
        x2 = 0,
        y2 = -scale,
        x3 = scale,
        y3 = scale,
        hback = -scale * 2,
    )
}

fn negedge_arrow(writer: &mut impl io::Write, wave_height: u32, color: Color) -> io::Result<()> {
    let scale = i64::from(wave_height / 6);

    write!(
        writer,
        r##"<path d="M{x1},{y1}L{x2},{y2}L{x3},{y3}H{hback}z" fill="{color}" stroke="none"/>"##,
        x1 = -scale,
        y1 = -scale,
        x2 = 0,
        y2 = scale,
        x3 = scale,
        y3 = -scale,
        hback = -scale * 2,
    )
}

impl<'a> AssembledFigure<'a> {
    /// Render a [`AssembledFigure`] into a `writer`.
    #[inline]
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        self.write_svg_with_options(writer, &Options::default())
    }

    /// Render a [`AssembledFigure`] into a `writer` with a set of options.
    pub fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Options,
    ) -> io::Result<()> {
        let Options {
            background,
            padding,
            spacing,
            header,
            footer,
            signal,
            reg: _,
            ..
        } = options;
        let SignalOptions {
            group_indicator,
            edge,
            ..
        } = signal;
        let PathOptions {
            signal_height,
            cycle_width,
            transition_offset: _,
        } = self.path_assemble_options;

        let signal_height = u32::from(signal_height);
        let cycle_width = u32::from(cycle_width);

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

        // Definitions
        write!(writer, "<defs>")?;
        if self.definitions.has_undefined {
            write!(
                writer,
                r##"<pattern id="x-bg" patternUnits="userSpaceOnUse" width="4" height="10" patternTransform="rotate(45)">"##,
            )?;

            if let Some(background) = options.undefined_background {
                write!(
                    writer,
                    r##"<rect x="0" y="0" width="4" height="10" fill="{background}"/>"##
                )?;
            }

            write!(
                writer,
                r##"<line x1="0" y="0" x2="0" y2="10" stroke="{color}" stroke-width="1"/></pattern>"##,
                color = signal.undefined_color
            )?;
        }

        if self.definitions.has_posedge_marker {
            write!(writer, r##"<g id="pei">"##)?;
            posedge_arrow(writer, signal_height, signal.path_color)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_negedge_marker {
            write!(writer, r##"<g id="nei">"##)?;
            negedge_arrow(writer, signal_height, signal.path_color)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_gaps {
            write!(writer, r##"<g id="gap">"##)?;
            gap(
                writer,
                self.path_assemble_options.signal_height,
                signal.gap_color,
                signal.gap_background_color,
            )?;
            write!(writer, r##"</g>"##)?;
        }

        write!(
            writer,
            r##"<g id="cl"><path fill="none" d="M0,0v{schema_height}" stroke-width="1" stroke-dasharray="2" stroke="{color}"/></g>"##,
            color = signal.hint_line_color,
            schema_height = dims.schema_height(),
        )?;
        write!(writer, "</defs>")?;

        // Background
        if let Some(background) = background {
            write!(
                writer,
                r##"<rect width="100%" height="100%" fill="{background}"/>"##
            )?;
        }

        // Header Text
        if let Some(title) = self.header_text {
            let title_font_size = header.font_size;
            let title_color = header.color;

            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{title_font_size}" fill="{title_color}" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = dims.header_x() + dims.header_width() / 2,
                y = dims.header_y() + dims.header_height() / 2,
                text = escape_str(title),
            )?;
        }

        // Top Cycle Enumeration Markers
        if let Some(cycle_marker) = self.top_cycle_marker {
            let start = cycle_marker.start();
            let every = cycle_marker.every();

            let marker_font_size = header.cycle_marker_fontsize;
            let marker_color = header.cycle_marker_color;
            let end = start + self.num_cycles;

            if every != 0 {
                write!(writer, "<g>")?;
                for offset in (start..end).step_by(every as usize) {
                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{marker_font_size}" fill="{marker_color}" letter-spacing="0"><tspan>{offset}</tspan></text>"##,
                        x = dims.schema_x()
                            + dims.cycle_width() * (offset - start)
                            + dims.cycle_width() / 2,
                        y = dims.header_y() + dims.header_height(),
                    )?;
                }
                write!(writer, "</g>")?;
            }
        }

        // Cycle Hint Lines
        write!(writer, "<g>")?;
        for i in 0..=self.num_cycles {
            write!(
                writer,
                r##"<use transform="translate({x},{y})" xlink:href="#cl"/>"##,
                x = dims.schema_x() + i * dims.cycle_width(),
                y = dims.schema_y(),
            )?;
        }
        write!(writer, "</g>")?;

        // Group Indicators
        if !self.group_markers.is_empty() {
            let label_font_size = group_indicator.label_fontsize;
            let label_color = group_indicator.label_color;

            write!(writer, "<g>")?;
            for group in self.group_markers.iter() {
                if group.is_empty() {
                    continue;
                }

                let depth = group.depth();
                let num_labels_below = self.amount_labels_below(depth);

                let height = group.len() * signal_height + (group.len() - 1) * spacing.line_to_line;
                let x = dims.grouping_x()
                    + if num_labels_below == 0 {
                        0
                    } else {
                        num_labels_below * group_indicator.label_height()
                            - group_indicator.label_spacing
                    }
                    + if depth == 0 {
                        0
                    } else {
                        depth * group_indicator.width + (depth - 1) * group_indicator.spacing
                    };
                let y = dims.schema_y()
                    + padding.schema_top
                    + if group.start() == 0 {
                        0
                    } else {
                        group.start() * signal_height + group.start() * spacing.line_to_line
                    };

                if let Some(label) = group.label() {
                    let x = x - group_indicator.label_fontsize / 2;

                    write!(
                        writer,
                        r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{label_font_size}" fill="{label_color}" letter-spacing="0" transform="rotate(270)"><tspan>{text}</tspan></text></g>"##,
                        y = y + height / 2,
                        text = escape_str(label),
                    )?;
                }

                write!(
                    writer,
                    r##"<path fill="none" d="M{x},{y}m{w},0c-3,0 -{w},1 -{w},{w}v{h}c0,3 1,{w} {w},{w}" stroke="{color}"/>"##,
                    color = group_indicator.color,
                    h = height - group_indicator.width * 2,
                    w = group_indicator.width,
                )?;
            }
            write!(writer, "</g>")?;
        }

        // Signal Lines
        write!(writer, "<g>")?;
        for (i, line) in self.lines.iter().enumerate() {
            if line.is_empty() {}

            let Ok(i) = u32::try_from(i) else {
                break;
            };

            let x = if dims.has_textbox() {
                dims.textbox_x()
            } else {
                dims.schema_x()
            };
            let y = dims.signal_top(i);

            write!(writer, r##"<g transform="translate({x},{y})">"##)?;

            if !line.text.is_empty() {
                let name_font_size = signal.name_font_size;
                let name_color = signal.name_color;

                write!(
                    writer,
                    r##"<g transform="translate(0,{y})"><text dominant-baseline="middle" font-family="{font_family}" font-size="{name_font_size}" fill="{name_color}" letter-spacing="0"><tspan>{text}</tspan></text></g>"##,
                    y = signal_height / 2,
                    text = escape_str(line.text),
                )?;
            }

            if dims.has_textbox() {
                write!(
                    writer,
                    r##"<g transform="translate({schema_x})">"##,
                    schema_x = dims.schema_x() - dims.textbox_x()
                )?;
                write_signal(&line.path, writer, options, self.hscale)?;
                write!(writer, r##"</g>"##)?;
            } else {
                write_signal(&line.path, writer, options, self.hscale)?;
            }

            write!(writer, r##"</g>"##)?;
        }
        write!(writer, "</g>")?;

        // Footer Text
        if let Some(footer_text) = self.footer_text {
            let footer_font_size = footer.font_size;
            let footer_color = footer.color;

            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{footer_font_size}" fill="{footer_color}" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = dims.footer_width() / 2,
                y = dims.footer_y() + dims.footer_height() / 2,
                text = escape_str(footer_text),
            )?;
        }

        // Bottom Cycle Enumeration Markers
        if let Some(cycle_marker) = self.bottom_cycle_marker {
            let start = cycle_marker.start();
            let every = cycle_marker.every();

            let marker_font_size = footer.cycle_marker_fontsize;
            let marker_color = footer.cycle_marker_color;

            let end = start + self.num_cycles;

            if every != 0 {
                write!(writer, "<g>")?;
                for offset in (start..end).step_by(every as usize) {
                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{marker_font_size}" fill="{marker_color}" letter-spacing="0"><tspan>{offset}</tspan></text>"##,
                        x = dims.schema_x()
                            + dims.cycle_width() * (offset - start)
                            + dims.cycle_width() / 2,
                        y = dims.footer_y()
                    )?;
                }
                write!(writer, "</g>")?;
            }
        }

        // Edge markers
        if !self.line_edge_markers.lines().is_empty() {
            let mut middles = Vec::with_capacity(self.line_edge_markers.lines().len());

            write!(writer, "<g>")?;
            for line_edge in self.line_edge_markers.lines() {
                middles.push(write_line_edge(
                    writer,
                    line_edge.clone(),
                    &dims,
                    options,
                    &font,
                )?);
            }

            for (line_edge, middle) in self
                .line_edge_markers
                .lines()
                .iter()
                .zip(middles.into_iter())
            {
                write_line_edge_markers(
                    writer,
                    line_edge.clone(),
                    middle,
                    &dims,
                    options,
                    &font,
                )?;
            }
            write!(writer, "</g>")?;
        }

        // Edge separate text markers
        if !self.line_edge_markers.text_nodes().is_empty() {
            write!(writer, "<g>")?;
            for text_node in self.line_edge_markers.text_nodes() {
                let text = text_node.text().to_string();
                let x = dims.schema_x() + text_node.at().x().width_offset(cycle_width);
                let y = dims.signal_top(text_node.at().y()) + signal_height / 2;

                write_edge_text(
                    writer,
                    (x.into(), y.into()),
                    &text,
                    edge.node_font_size,
                    edge.node_text_color,
                    edge.node_background_color,
                    &font,
                )?;
            }
            write!(writer, "</g>")?;
        }

        write!(writer, "</svg>")?;

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
    options: &Options,
    hscale: u16,
) -> io::Result<()> {
    let PathOptions {
        signal_height,
        cycle_width,
        transition_offset: _,
    } = wave_path.options();

    let signal_height = u32::from(*signal_height);
    let cycle_width = u32::from(*cycle_width);

    for segment in wave_path.segments() {
        let x = segment.x();
        let y = segment.y();

        write!(writer, r##"<path fill=""##)?;
        match segment.background() {
            Some(PathSegmentBackground::B2) => write!(writer, "{}", options.backgrounds[0])?,
            Some(PathSegmentBackground::B3) => write!(writer, "{}", options.backgrounds[1])?,
            Some(PathSegmentBackground::B4) => write!(writer, "{}", options.backgrounds[2])?,
            Some(PathSegmentBackground::B5) => write!(writer, "{}", options.backgrounds[3])?,
            Some(PathSegmentBackground::B6) => write!(writer, "{}", options.backgrounds[4])?,
            Some(PathSegmentBackground::B7) => write!(writer, "{}", options.backgrounds[5])?,
            Some(PathSegmentBackground::B8) => write!(writer, "{}", options.backgrounds[6])?,
            Some(PathSegmentBackground::B9) => write!(writer, "{}", options.backgrounds[7])?,
            Some(PathSegmentBackground::Undefined) => write!(writer, "url(#x-bg)")?,
            None => write!(writer, "none")?,
        }
        write!(writer, r#"" d=""#)?;
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
        write!(
            writer,
            r##"" stroke-width="1" stroke="{path_color}"/>"##,
            path_color = options.signal.path_color
        )?;

        if let Some(marker_text) = segment.marker_text() {
            write!(
                writer,
                r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" fill="{color}" letter-spacing="0"><tspan>{text}</tspan></text></g>"##,
                font_family = Font::default()
                    .get_font_family_name()
                    .as_ref()
                    .map(|s| &s[..])
                    .unwrap_or("Helvetica"),
                font_size = options.signal.marker_font_size,
                text = marker_text,
                color = options.signal.marker_color,
                x = segment.x() + segment.width() / 2,
                y = signal_height / 2,
            )?;
        }

        for clock_edge_marker in segment.clock_edge_markers() {
            let x = clock_edge_marker
                .at()
                .width_offset(cycle_width * u32::from(hscale));
            let y = signal_height / 2;

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
            let x = gap.width_offset(cycle_width * u32::from(hscale));
            let y = signal_height / 2;

            write!(
                writer,
                r##"<use transform="translate({x},{y})" xlink:href="#gap"/>"##,
            )?;
        }
    }

    Ok(())
}
