use std::io;

use crate::path::{ClockEdgeMarker, PathCommand, PathSegmentBackground};
use crate::{ClockEdge, WaveOptions};

use super::path::AssembledWavePath;
use super::AssembledFigure;

#[derive(Debug, Clone)]
pub struct FigurePadding {
    pub figure_top: u32,
    pub figure_bottom: u32,
    pub figure_left: u32,
    pub figure_right: u32,

    pub schema_top: u32,
    pub schema_bottom: u32,
}

#[derive(Debug, Clone)]
pub struct FigureSpacing {
    pub textbox_to_schema: u32,
    pub groupbox_to_textbox: u32,
    pub line_to_line: u32,
}

#[derive(Debug, Clone)]
pub struct GroupIndicatorDimension {
    width: u32,

    spacing: u32,

    label_spacing: u32,
    label_fontsize: u32,
}

#[derive(Debug, Clone)]
pub struct HeaderOptions {
    font_size: u32,
    height: u32,
}

#[derive(Debug, Clone)]
pub struct FooterOptions {
    font_size: u32,
    height: u32,
}

impl Default for HeaderOptions {
    fn default() -> Self {
        Self { font_size: 24, height: 32 }
    }
}

impl Default for FooterOptions {
    fn default() -> Self {
        Self { font_size: 24, height: 32 }
    }
}

impl GroupIndicatorDimension {
    fn label_height(&self) -> u32 {
        self.label_spacing + self.label_fontsize
    }
}

impl Default for GroupIndicatorDimension {
    fn default() -> Self {
        Self {
            width: 4,
            spacing: 4,

            label_spacing: 5,
            label_fontsize: 14,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub font_size: u32,
    pub paddings: FigurePadding,
    pub spacings: FigureSpacing,
    pub header: HeaderOptions,
    pub footer: FooterOptions,
    pub wave_dimensions: WaveOptions,
    pub group_indicator_dimensions: GroupIndicatorDimension,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 14,
            paddings: FigurePadding::default(),
            spacings: FigureSpacing::default(),
            header: HeaderOptions::default(),
            footer: FooterOptions::default(),
            wave_dimensions: WaveOptions::default(),
            group_indicator_dimensions: GroupIndicatorDimension::default(),
        }
    }
}

impl Default for FigurePadding {
    fn default() -> Self {
        Self {
            figure_top: 8,
            figure_bottom: 8,
            figure_left: 8,
            figure_right: 8,

            schema_top: 8,
            schema_bottom: 8,
        }
    }
}

impl Default for FigureSpacing {
    fn default() -> Self {
        Self {
            groupbox_to_textbox: 8,
            textbox_to_schema: 8,
            line_to_line: 8,
        }
    }
}

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

impl<'a> ToSvg for AssembledFigure<'a> {
    type Options = RenderOptions;

    fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Self::Options,
    ) -> io::Result<()> {
        let RenderOptions {
            font_size,
            paddings,
            spacings,
            wave_dimensions,
            group_indicator_dimensions,
            header,
            footer,
        } = options;

        let face =
            // ttf_parser::Face::parse(include_bytes!("../JetBrainsMono-Medium.ttf"), 0).unwrap();
            // ttf_parser::Face::parse(include_bytes!("/usr/share/fonts/noto/NotoSansMono-Regular.ttf"), 0).unwrap();
            ttf_parser::Face::parse(include_bytes!("../helvetica.ttf"), 0).unwrap();

        let font_family = get_font_family_name(&face).unwrap_or_else(|| "monospace".to_string());

        let header_height = if self.title.is_some() { header.height } else { 0 };

        let footer_height = if self.footer.is_some() { footer.height } else { 0 };

        let has_textbox = !self.lines.iter().all(|line| line.text.is_empty());
        let textbox_width = has_textbox.then(|| {
            self.lines
                .iter()
                .map(|line| get_text_width(line.text, &face, options.font_size))
                .max()
                .unwrap_or_default()
        });

        let schema_width = self.num_cycles * u32::from(wave_dimensions.cycle_width);
        let schema_height = if self.lines.len() == 0 {
            0
        } else {
            let num_lines = self.lines.len() as u32;

            paddings.schema_top
                + paddings.schema_bottom
                + spacings.line_to_line * (num_lines - 1)
                + u32::from(wave_dimensions.wave_height) * num_lines
        };

        let groupbox_width = if self.max_group_depth == 0 {
            None
        } else {
            Some(
                self.max_group_depth * group_indicator_dimensions.width
                    + (self.max_group_depth - 1) * group_indicator_dimensions.spacing
                    + self.group_label_at_depth.iter().filter(|x| **x).count() as u32
                        * group_indicator_dimensions.label_height(),
            )
        };

        let figure_width = paddings.figure_left
            + groupbox_width.map_or(0, |w| w + spacings.groupbox_to_textbox)
            + textbox_width.map_or(0, |w| w + spacings.textbox_to_schema)
            + schema_width
            + paddings.figure_right;
        let figure_height = paddings.figure_top
            + header_height
            + schema_height
            + footer_height
            + paddings.figure_bottom;

        let (textbox_x, schema_x) = match (groupbox_width, textbox_width) {
            (Some(groupbox_width), Some(textbox_width)) => {
                let textbox_x = groupbox_width + spacings.groupbox_to_textbox;
                (
                    Some(textbox_x),
                    textbox_x + textbox_width + spacings.textbox_to_schema,
                )
            }
            (Some(groupbox_width), None) => (None, groupbox_width + spacings.groupbox_to_textbox),
            (None, Some(textbox_width)) => (Some(0), textbox_width + spacings.textbox_to_schema),
            (None, None) => (None, 0),
        };

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}" overflow="hidden" width="{figure_width}" height="{figure_height}">"#,
        )?;

        // Define the cycle-to-cycle background hint lines
        write!(writer, "<defs>")?;
        if self.has_undefined {
            write!(
                writer,
                r##"<pattern id="x-bg" patternUnits="userSpaceOnUse" width="4" height="10" patternTransform="rotate(45)"><line x1="0" y="0" x2="0" y2="10" stroke="#000" stroke-width="1"/></pattern>"##
            )?;
        }
        {
            // (-4, 4) -> (0, -4) -> (4, 4)
            let [x1, y1, x2, y2, x3, y3] = [-4, 4, 4, -8, 4, 8];

            write!(
                writer,
                r##"<g id="pei"><path d="M{x1},{y1}l{x2},{y2}l{x3},{y3}h-8z" fill="#000" stroke="none"/></g>"##,
            )?;
        }
        {
            // (-4, -4) -> (0, 4) -> (4, -4)
            let [x1, y1, x2, y2, x3, y3] = [-4, -4, 4, 8, 4, -8];

            write!(
                writer,
                r##"<g id="nei"><path d="M{x1},{y1}l{x2},{y2}l{x3},{y3}h-8z" fill="#000" stroke="none"/></g>"##,
            )?;
        }
        {
            let wave_height = f32::from(options.wave_dimensions.wave_height);

            let a: f32 = 8.0;
            let b = wave_height / 2.0 + 6.0;

            const DISTANCE: f32 = 4.0;

            let start = (-a, b);

            let control_1 = (start.0 + a / 2.0, start.1);

            let rad = (-2.0 * b / a).atan();
            let control_2 = (rad.cos() * a / -2.0, rad.sin() * a / -2.0);

            let end = (a, -b);

            let control_3 = (end.0 - a / 2.0, end.1);

            write!(
                writer,
                r##"<g id="gap"><path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}H{rp1x}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}H{lp1x}z" fill="#fff" stroke="none"/><path d="M{lp1x},{lp1y}C{lp2x},{lp2y} {lp3x},{lp3y} {lp4x},{lp4y}S{lp5x},{lp5y} {lp6x},{lp6y}" fill="none" stroke="#000" stroke-width="1"/><path d="M{rp1x},{rp1y}C{rp2x},{rp2y} {rp3x},{rp3y} {rp4x},{rp4y}S{rp5x},{rp5y} {rp6x},{rp6y}" fill="none" stroke="#000" stroke-width="1"/></g>"##,
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
            )?;
        }
        write!(
            writer,
            r##"<g id="cl"><path fill="none" d="M0,0v{schema_height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC"/></g>"##
        )?;
        write!(writer, "</defs>")?;

        // Figure container
        write!(
            writer,
            r##"<g transform="translate({padding_x},{padding_y})">"##,
            padding_x = paddings.figure_left,
            padding_y = paddings.figure_top,
        )?;

        if let Some(title) = self.title {
            let title_font_size = header.font_size;
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{title_font_size}" letter-spacing="0"><tspan>{title}</tspan></text>"##,
                x = (figure_width - paddings.figure_left - paddings.figure_right) / 2,
                y = header_height / 2
            )?;
        }

        write!(
            writer,
            r##"<g transform="translate({schema_x},{header_height})">"##,
        )?;
        for i in 0..=u64::from(self.num_cycles) {
            write!(
                writer,
                r##"<use transform="translate({x})" xlink:href="#cl"/>"##,
                x = i * u64::from(wave_dimensions.cycle_width)
            )?;
        }
        write!(writer, r##"</g>"##)?;

        for group in self.groups.iter() {
            if group.is_empty() {
                continue;
            }

            let height = group.len() * u32::from(wave_dimensions.wave_height)
                + (group.len() - 1) * spacings.line_to_line;
            let x = self.amount_labels_before(group.depth + 1)
                * group_indicator_dimensions.label_height()
                + if group.depth == 0 {
                    0
                } else {
                    group.depth * group_indicator_dimensions.width
                };
            let y = header_height
                + paddings.schema_top
                + if group.start == 0 {
                    0
                } else {
                    group.start * u32::from(wave_dimensions.wave_height)
                        + group.start * spacings.line_to_line
                };

            if let Some(label) = group.label {
                let x = if group.depth == 0 {
                    0
                } else {
                    self.amount_labels_before(group.depth)
                        * group_indicator_dimensions.label_height()
                        + group.depth * group_indicator_dimensions.width
                };

                // let label_width = get_text_width(label, &face, 8);
                write!(
                    writer,
                    r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0" transform="rotate(270)"><tspan>{label}</tspan></text></g>"##,
                    font_size = group_indicator_dimensions.label_fontsize,
                    x = x + group_indicator_dimensions.label_spacing + 2,
                    y = y + height / 2,
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

            let x = textbox_x.unwrap_or(schema_x);
            let y = header_height
                + paddings.schema_top
                + if i == 0 {
                    0
                } else {
                    (u32::from(wave_dimensions.wave_height) + spacings.line_to_line) * i
                };

            write!(writer, r##"<g transform="translate({x},{y})">"##)?;

            if !line.text.is_empty() {
                write!(
                    writer,
                    r##"<g transform="translate(0,{y})"><text dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}" letter-spacing="0"><tspan>{text}</tspan></text></g>"##,
                    font_size = font_size,
                    y = wave_dimensions.wave_height / 2,
                    text = line.text,
                )?;
            }

            write!(
                writer,
                r##"<g transform="translate({schema_x})">"##,
                schema_x = textbox_x.map_or(0, |textbox_x| schema_x - textbox_x)
            )?;
            line.path.write_svg_with_options(writer, &wave_dimensions)?;

            write!(writer, r##"</g>"##)?;

            write!(writer, r##"</g>"##)?;
        }

        if let Some(footer_text) = self.footer {
            let footer_font_size = footer.font_size;
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{footer_font_size}" letter-spacing="0"><tspan>{footer_text}</tspan></text>"##,
                x = (figure_width - paddings.figure_left - paddings.figure_right) / 2,
                y = header_height + schema_height + footer_height / 2
            )?;
        }

        write!(writer, "</g></svg>")?;

        Ok(())
    }
}

impl ToSvg for AssembledWavePath {
    type Options = WaveOptions;

    fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Self::Options,
    ) -> io::Result<()> {
        for segment in self.segments() {
            let fill = match segment.background() {
                Some(PathSegmentBackground::Index(2)) => "#ff4040",
                Some(PathSegmentBackground::Index(3)) => "#5499C7",
                Some(PathSegmentBackground::Index(4)) => "#58D68D",
                Some(PathSegmentBackground::Index(5)) => "#A569BD",
                Some(PathSegmentBackground::Index(_)) => unimplemented!(),
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
                    font_family = options.font_family,
                    font_size = options.font_size,
                    text = marker_text,
                    x = segment.x() + segment.width() / 2,
                    y = options.wave_height / 2,
                )?;
            }

            for ClockEdgeMarker { x, edge } in segment.clock_edge_markers() {
                let x = *x;
                let y = u32::from(options.wave_height) / 2;

                match edge {
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
                let x = u32::from(options.cycle_width) * *gap + u32::from(options.cycle_width) / 2;
                let y = u32::from(options.wave_height) / 2;

                write!(
                    writer,
                    r##"<use transform="translate({x},{y})" xlink:href="#gap"/>"##,
                )?;
            }
        }

        Ok(())
    }
}

fn get_text_width(s: &str, face: &ttf_parser::Face, font_size: u32) -> u32 {
    let width = s
        .chars()
        .map(|c| {
            face.glyph_index(c).map_or_else(
                || {
                    // warn!("[WARNING]: Failed to get glyph for '{c}'");
                    0
                },
                |g| {
                    u32::from(face.glyph_hor_advance(g).unwrap_or_else(|| {
                        // warn!(
                        //     "[WARNING]: Failed to get length for glyph '{}' that represents character '{c}'",
                        //     face.glyph_name(g).unwrap_or(&c.to_string())
                        // );
                        0
                    }))
                },
            )
        })
        .sum::<u32>();

    let width = f64::from(width);

    // NOTE: Face::units_per_em guarantees the value to be non-zero. So this should never
    // generate a divide-by-zero error.
    let pts_per_em = f64::from(font_size) / f64::from(face.units_per_em());
    let width = width * pts_per_em;

    width.ceil() as u32
}

fn name_to_string(name: ttf_parser::name::Name) -> Option<String> {
    if !name.is_unicode() {
        return None;
    }

    // Invalid UTF16 check
    if name.name.len() % 2 != 0 {
        return None;
    }

    let utf16_bytes = name
        .name
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<u16>>();

    String::from_utf16(&utf16_bytes).ok()
}

fn get_font_family_name(face: &ttf_parser::Face) -> Option<String> {
    for item in face.names() {
        if item.name_id == 1 {
            return name_to_string(item);
        }
    }

    None
}
