use std::io;
use std::marker::PhantomData;

use crate::path::{ClockEdgeMarker, PathCommand, PathSegmentBackground};
use crate::{ClockEdge, CycleMarker, WaveOptions};

use super::path::AssembledWavePath;
use super::AssembledFigure;

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

    cycle_marker_height: u32,
    cycle_marker_font_size: u32,
}

#[derive(Debug, Clone)]
pub struct FooterOptions {
    font_size: u32,
    height: u32,

    cycle_marker_height: u32,
    cycle_marker_font_size: u32,
}

struct SvgDimensions<'a> {
    figure: &'a AssembledFigure<'a>,
    options: &'a RenderOptions,
    textbox_width: Option<u32>,
}

impl<'a> SvgDimensions<'a> {
    fn new(figure: &'a AssembledFigure<'a>, font: Font, options: &'a RenderOptions) -> Self {
        let has_textbox = !figure.lines.iter().all(|line| line.text.is_empty());
        let textbox_width = has_textbox.then(|| {
            figure
                .lines
                .iter()
                .map(|line| font.get_text_width(line.text, options.font_size))
                .max()
                .unwrap_or_default()
        });

        Self {
            figure,
            options,
            textbox_width,
        }
    }

    fn inner_width(&self) -> u32 {
        let RenderOptions { spacings, .. } = self.options;

        let mut width = self.schema_width();

        if self.has_grouping() {
            width += self.grouping_width() + spacings.groupbox_to_textbox;
        }

        if self.has_textbox() {
            width += self.textbox_width() + spacings.textbox_to_schema;
        }

        width
    }

    #[inline]
    fn inner_x(&self) -> u32 {
        self.options.paddings.figure_left
    }

    #[inline]
    fn figure_width(&self) -> u32 {
        let RenderOptions { paddings, .. } = self.options;
        paddings.figure_left + paddings.figure_right + self.inner_width()
    }

    #[inline]
    fn figure_height(&self) -> u32 {
        let RenderOptions { paddings, .. } = self.options;

        paddings.figure_top
            + self.header_height()
            + self.schema_height()
            + self.footer_height()
            + paddings.figure_bottom
    }

    #[inline]
    fn header_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    fn header_height(&self) -> u32 {
        let RenderOptions { header, .. } = self.options;

        let mut height = 0;

        if self.figure.title.is_some() {
            height += header.height;
        }

        if self.figure.top_cycle_marker.is_some() {
            height += header.cycle_marker_height;
        }

        height
    }

    // #[inline]
    // fn header_x(&self) -> u32 {
    //     self.options.paddings.figure_left
    // }

    #[inline]
    fn header_y(&self) -> u32 {
        self.options.paddings.figure_top
    }

    #[inline]
    fn footer_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    fn footer_height(&self) -> u32 {
        let RenderOptions { footer, .. } = self.options;

        let mut height = 0;

        if self.figure.footer.is_some() {
            height += footer.height;
        }

        if self.figure.bottom_cycle_marker.is_some() {
            height += footer.cycle_marker_height;
        }

        height
    }

    // #[inline]
    // fn footer_x(&self) -> u32 {
    //     self.options.paddings.figure_left
    // }

    #[inline]
    fn footer_y(&self) -> u32 {
        self.schema_y() + self.schema_height()
    }

    fn has_textbox(&self) -> bool {
        self.figure.lines.iter().any(|line| !line.text.is_empty())
    }
    fn textbox_width(&self) -> u32 {
        self.textbox_width.unwrap_or(0)
    }

    // #[inline]
    // fn textbox_height(&self) -> u32 {
    //     self.schema_height()
    // }

    #[inline]
    fn textbox_x(&self) -> u32 {
        let mut x = self.grouping_x();

        if self.has_grouping() {
            x += self.grouping_width() + self.options.spacings.groupbox_to_textbox;
        }

        x
    }

    // #[inline]
    // fn textbox_y(&self) -> u32 {
    //     self.header_y() + self.header_height()
    // }

    #[inline]
    fn has_grouping(&self) -> bool {
        self.figure.max_group_depth != 0
    }

    #[inline]
    fn grouping_x(&self) -> u32 {
        self.inner_x()
    }

    // #[inline]
    // fn grouping_y(&self) -> u32 {
    //     self.header_y() + self.header_height()
    // }

    fn grouping_width(&self) -> u32 {
        let max_group_depth = self.figure.max_group_depth;

        if max_group_depth == 0 {
            return 0;
        }

        let RenderOptions {
            group_indicator_dimensions,
            ..
        } = self.options;

        let sum_indicator_widths = max_group_depth * group_indicator_dimensions.width;
        let spacing = (max_group_depth - 1) * group_indicator_dimensions.spacing;
        let label_widths = self
            .figure
            .group_label_at_depth
            .iter()
            .filter(|x| **x)
            .count() as u32
            * group_indicator_dimensions.label_height();

        sum_indicator_widths + spacing + label_widths
    }

    // #[inline]
    // fn grouping_height(&self) -> u32 {
    //     self.schema_height()
    // }

    #[inline]
    fn schema_x(&self) -> u32 {
        let mut x = self.textbox_x();

        if self.has_textbox() {
            x += self.textbox_width() + self.options.spacings.textbox_to_schema;
        }

        x
    }

    #[inline]
    fn schema_y(&self) -> u32 {
        self.header_y() + self.header_height()
    }

    #[inline]
    fn schema_width(&self) -> u32 {
        self.figure.num_cycles * self.cycle_width()
    }

    fn schema_height(&self) -> u32 {
        if self.figure.lines.len() == 0 {
            return 0;
        }

        let RenderOptions {
            paddings, spacings, ..
        } = self.options;

        let num_lines = self.num_lines();

        paddings.schema_top
            + paddings.schema_bottom
            + spacings.line_to_line * (num_lines - 1)
            + self.wave_height() * num_lines
    }

    #[inline]
    fn cycle_width(&self) -> u32 {
        (self.figure.hscale * self.options.wave_dimensions.cycle_width).into()
    }

    #[inline]
    fn wave_height(&self) -> u32 {
        self.options.wave_dimensions.wave_height.into()
    }

    #[inline]
    fn num_lines(&self) -> u32 {
        self.figure.lines.len() as u32
    }
}

impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            font_size: 24,
            height: 32,
            cycle_marker_height: 12,
            cycle_marker_font_size: 12,
        }
    }
}

impl Default for FooterOptions {
    fn default() -> Self {
        Self {
            font_size: 24,
            height: 32,
            cycle_marker_height: 12,
            cycle_marker_font_size: 12,
        }
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

impl GroupIndicatorDimension {
    fn label_height(&self) -> u32 {
        self.label_spacing + self.label_fontsize
    }
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

#[derive(Debug, Clone, Copy)]
pub enum Font<'a> {
    #[cfg(feature = "embed_font")]
    FontFace(&'a ttf_parser::Face<'a>),
    #[allow(unused)]
    HelveticaLookUpTable(PhantomData<&'a ()>),
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

        #[cfg(feature = "embed_font")]
        let face = ttf_parser::Face::parse(include_bytes!("../helvetica.ttf"), 0).unwrap();
        #[cfg(feature = "embed_font")]
        let font = Font::FontFace(&face);

        #[cfg(not(feature = "embed_font"))]
        let font = Font::HelveticaLookUpTable(PhantomData::default());

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
            posedge_arrow(writer, wave_dimensions.wave_height)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_negedge_marker {
            write!(writer, r##"<g id="nei">"##)?;
            negedge_arrow(writer, wave_dimensions.wave_height)?;
            write!(writer, r##"</g>"##)?;
        }

        if self.definitions.has_gap {
            write!(writer, r##"<g id="gap">"##)?;
            gap(writer, wave_dimensions.wave_height)?;
            write!(writer, r##"</g>"##)?;
        }

        // Define the cycle-to-cycle background hint lines
        write!(
            writer,
            r##"<g id="cl"><path fill="none" d="M0,0v{schema_height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC"/></g>"##,
            schema_height = dims.schema_height(),
        )?;
        write!(writer, "</defs>")?;

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
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{title_font_size}" letter-spacing="0"><tspan>{title}</tspan></text>"##,
                x = dims.header_width() / 2,
                y = dims.header_height() / 2
            )?;
        }
        if let Some(CycleMarker { start, every }) = self.top_cycle_marker {
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
                        y = dims.header_height()
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

            let height = group.len() * u32::from(wave_dimensions.wave_height)
                + (group.len() - 1) * spacings.line_to_line;
            let x = self.amount_labels_before(group.depth + 1)
                * group_indicator_dimensions.label_height()
                + if group.depth == 0 {
                    0
                } else {
                    group.depth * group_indicator_dimensions.width
                };
            let y = dims.header_height()
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

            let x = dims
                .has_textbox()
                .then_some(dims.textbox_x())
                .unwrap_or(dims.schema_x());
            let y = dims.header_height()
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
                r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{footer_font_size}" letter-spacing="0"><tspan>{footer_text}</tspan></text>"##,
                x = dims.footer_width() / 2,
                y = dims.footer_y() + dims.footer_height() / 2
            )?;
        }
        if let Some(CycleMarker { start, every }) = self.bottom_cycle_marker {
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
    wave_path: &AssembledWavePath,
    writer: &mut impl io::Write,
    options: &WaveOptions,
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
                    PathCommand::DashedLineHorizontal(dx) => draw_dashed_horizontal_line(writer, *dx),
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
            let x = gap.width_offset(u32::from(options.cycle_width * hscale))
                + u32::from(options.cycle_width * hscale) / 2;
            let y = u32::from(options.wave_height) / 2;

            write!(
                writer,
                r##"<use transform="translate({x},{y})" xlink:href="#gap"/>"##,
            )?;
        }
    }

    Ok(())
}

impl<'a> Font<'a> {
    fn units_per_em(&self) -> u16 {
        match self {
            Self::HelveticaLookUpTable(_) => 2048,
            #[cfg(feature = "embed_font")]
            Self::FontFace(face) => face.units_per_em(),
        }
    }

    fn get_text_width(&self, s: &'_ str, font_size: u32) -> u32 {
        let width = match self {
            Self::HelveticaLookUpTable(_) => {
                static ADVANCE_LUT: [u16; 128] = [
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 569, 569, 0, 0, 569, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 569, 569, 727, 1139, 1139, 1821, 1366, 391, 682, 682,
                    797, 1196, 569, 682, 569, 569, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139,
                    1139, 1139, 569, 569, 1196, 1196, 1196, 1139, 2079, 1366, 1366, 1479, 1479,
                    1366, 1251, 1593, 1479, 569, 1024, 1366, 1139, 1706, 1479, 1593, 1366, 1593,
                    1479, 1366, 1251, 1479, 1366, 1933, 1366, 1366, 1251, 569, 569, 569, 961, 1139,
                    682, 1139, 1139, 1024, 1139, 1139, 569, 1139, 1139, 455, 455, 1024, 455, 1706,
                    1139, 1139, 1139, 1139, 682, 1024, 569, 1139, 1024, 1479, 1024, 1024, 1024,
                    684, 532, 684, 1196, 0,
                ];

                s.chars()
                    .map(|c| {
                        if c.is_ascii() {
                            u32::from(ADVANCE_LUT[c as usize])
                        } else {
                            2052
                        }
                    })
                    .sum::<u32>()
            }
            #[cfg(feature = "embed_font")]
            Self::FontFace(face) => s
                .chars()
                .map(|c| {
                    face.glyph_index(c)
                        .and_then(|g| face.glyph_hor_advance(g))
                        .map(u32::from)
                        .unwrap_or(0)
                })
                .sum::<u32>(),
        };

        let width = f64::from(width);

        // NOTE: Face::units_per_em guarantees the value to be non-zero. So this should never
        // generate a divide-by-zero error.
        let pts_per_em = f64::from(font_size) / f64::from(self.units_per_em());
        let width = width * pts_per_em;

        width.ceil() as u32
    }

    fn get_font_family_name(&self) -> Option<String> {
        match self {
            Self::HelveticaLookUpTable(_) => Some("helvetica".to_string()),
            #[cfg(feature = "embed_font")]
            Self::FontFace(face) => face
                .names()
                .into_iter()
                .find(|item| item.name_id == 1)
                .map_or(None, |name| {
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
                }),
        }
    }
}

#[ignore]
#[test]
#[cfg(all(feature = "gen_lut", feature = "embed_font"))]
fn generate_lookup_table() {
    let face = ttf_parser::Face::parse(include_bytes!("../helvetica.ttf"), 0).unwrap();

    println!("[");
    for c in 0..128u8 {
        let c = c as char;
        println!(
            "\t{},",
            face.glyph_index(c)
                .and_then(|g| face.glyph_hor_advance(g))
                .map(u32::from)
                .unwrap_or(0)
        );
    }
    println!("]");
    println!("units_per_em: {:?}", face.units_per_em());
    println!("Rest Advance: {:?}", face.global_bounding_box().width());

    assert!(false);
}
