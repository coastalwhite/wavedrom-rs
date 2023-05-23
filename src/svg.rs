use std::io;

use crate::path::PathCommand;
use crate::rect::Rect;
use crate::{WaveDimension, WaveGroup};

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
    padding_top: u32,
    padding_bottom: u32,

    width: u32,

    spacing: u32,

    label_spacing: u32,
    label_fontsize: u32,
}

pub struct SvgBoundingBoxes {
    figure: Rect,
    group_indicators: Rect,
    textbox: Rect,
    schema: Rect,
}

impl Default for GroupIndicatorDimension {
    fn default() -> Self {
        Self {
            padding_top: 0,
            padding_bottom: 0,
            width: 4,
            spacing: 4,

            label_spacing: 4,
            label_fontsize: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub font_size: u32,
    pub paddings: FigurePadding,
    pub spacings: FigureSpacing,
    pub wave_dimensions: WaveDimension,
    pub group_indicator_dimensions: GroupIndicatorDimension,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 10,
            paddings: FigurePadding::default(),
            spacings: FigureSpacing::default(),
            wave_dimensions: WaveDimension::default(),
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
            groupbox_to_textbox: 16,
            textbox_to_schema: 16,
            line_to_line: 16,
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
        } = options;

        let face =
            // ttf_parser::Face::parse(include_bytes!("../JetBrainsMono-Medium.ttf"), 0).unwrap();
            ttf_parser::Face::parse(include_bytes!("/usr/share/fonts/noto/NotoSansMono-Regular.ttf"), 0).unwrap();

        let font_family = get_font_family_name(&face).unwrap_or_else(|| "monospace".to_string());

        let textbox_width = self
            .lines
            .iter()
            .map(|line| get_text_width(line.text, &face, options.font_size))
            .max()
            .unwrap_or_default();

        // Error when there are too many cycles
        let schema_width = self.num_cycles * u32::from(wave_dimensions.cycle_width);

        let schema_height = if self.lines.len() == 0 {
            0
        } else {
            let num_lines = u32::try_from(self.lines.len()).map_err(|_| ()).unwrap();

            paddings.schema_top
                + paddings.schema_bottom
                + spacings.line_to_line * (num_lines - 1)
                + u32::from(wave_dimensions.wave_height) * num_lines
        };

        let groupbox_width = (self.max_group_depth > 0).then(|| {
            self.max_group_depth * group_indicator_dimensions.width
                + (self.max_group_depth - 1) * group_indicator_dimensions.spacing
                + self.group_label_at_depth.iter().filter(|x| **x).count() as u32
                    * (group_indicator_dimensions.label_spacing
                        + group_indicator_dimensions.label_fontsize)
        });

        let figure_width = paddings.figure_left
            + paddings.figure_right
            + groupbox_width.map_or(0, |w| w + spacings.groupbox_to_textbox)
            + textbox_width
            + schema_width
            + spacings.textbox_to_schema;
        let figure_height = paddings.figure_top + paddings.figure_bottom + schema_height;

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}">"#,
        )?;

        write!(
            writer,
            r##"<defs><g id="cl"><path fill="none" d="M0,0v{schema_height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC" /></g></defs>"##,
        )?;

        write!(
            writer,
            r##"<g transform="translate({padding_x},{padding_y})">"##,
            padding_x = paddings.figure_left,
            padding_y = paddings.figure_top,
        )?;

        let textbox_x = groupbox_width.map_or(0, |w| w + spacings.groupbox_to_textbox);
        let schema_x = textbox_x + textbox_width + spacings.textbox_to_schema;

        write!(writer, r##"<g transform="translate({schema_x})">"##,)?;
        for i in 0..=u64::from(self.num_cycles) {
            write!(
                writer,
                r##"<use transform="translate({x})" xlink:href="#cl" />"##,
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
            let x = self
                .group_label_at_depth
                .iter()
                .take((group.depth + 1) as usize)
                .filter(|x| **x)
                .count() as u32
                * (group_indicator_dimensions.label_spacing
                    + group_indicator_dimensions.label_fontsize)
                + if group.depth == 0 {
                    0
                } else {
                    group.depth * group_indicator_dimensions.width
                };
            let y = paddings.schema_top
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
                    self.group_label_at_depth
                        .iter()
                        .take(group.depth as usize)
                        .filter(|x| **x)
                        .count() as u32
                        * (group_indicator_dimensions.label_spacing
                            + group_indicator_dimensions.label_fontsize)
                        + group.depth * group_indicator_dimensions.width
                };

                // let label_width = get_text_width(label, &face, 8);
                write!(
                    writer,
                    r##"<g transform="translate({x},{y})"><text text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{font_size}px" letter-spacing="0" transform="rotate(270)"><tspan>{label}</tspan></text></g>"##,
                    font_size = group_indicator_dimensions.label_fontsize,
                    x = x + group_indicator_dimensions.label_spacing + 2,
                    y = y + height / 2,
                    // y = wave_dimensions.wave_height / 2,
                )?;
            }

            write!(
                writer,
                r##"<path fill="none" d="M{x},{y}m{w},0c-3,0 -{w},1 -{w},{w}v{height}c0,3 1,{w} {w},{w}" stroke="#000"/>"##,
                height = height - group_indicator_dimensions.width * 2,
                w = group_indicator_dimensions.width,
            )?;
        }

        for (i, line) in self.lines.iter().enumerate() {
            let Ok(i) = u32::try_from(i) else {
                break;
            };

            write!(
                writer,
                r##"<g transform="translate({textbox_x},{y})">"##,
                y = paddings.schema_top
                    + if i == 0 {
                        0
                    } else {
                        u32::from(wave_dimensions.wave_height) * i + spacings.line_to_line * i
                    }
            )?;

            write!(
                writer,
                r##"<text dominant-baseline="middle" font-family="{font_family}" y="{y}pt" font-size="{font_size}px" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                font_size = font_size,
                y = wave_dimensions.wave_height / 2,
                text = line.text,
            )?;

            write!(
                writer,
                r##"<g transform="translate({schema_x})">"##,
                schema_x = schema_x - textbox_x
            )?;
            line.path
                .render_with_options(&wave_dimensions)
                .write_svg(writer)?;
            write!(writer, r##"</g>"##)?;

            write!(writer, r##"</g>"##)?;
        }

        write!(writer, "</g></svg>")?;

        Ok(())
    }
}

impl ToSvg for AssembledWavePath {
    type Options = ();

    fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        _: &Self::Options,
    ) -> io::Result<()> {
        for segment in self.segments() {
            let fill = segment
                .data_index()
                .map_or("none", |data_index| match data_index {
                    0 => "#ff4040",
                    1 => "#5499C7",
                    2 => "#58D68D",
                    3 => "#A569BD",
                    _ => unimplemented!(),
                });

            let x = segment.x();
            let y = segment.y();

            write!(writer, r##"<path fill="{fill}" d=""##)?;
            write!(writer, "M{x},{y}")?;
            for action in segment.actions() {
                match action {
                    PathCommand::LineVerticalNoStroke(dy) => write!(writer, "v{dy}"),
                    PathCommand::LineHorizontal(dx) => write!(writer, "h{dx}"),
                    PathCommand::Line(dx, dy) => write!(writer, "l{dx},{dy}"),
                }?
            }

            // If there is a `no_stroke` element, we need to divide up the filling and the
            // stroking.
            if !segment.is_fully_stroked() && !segment.is_open() {
                write!(writer, r##"z" stroke="none"/>"##)?;

                write!(writer, r##"<path fill="none" d=""##)?;
                write!(writer, "M{x},{y}")?;
                for action in segment.actions() {
                    match action {
                        PathCommand::LineVerticalNoStroke(dy) => write!(writer, "m0,{dy}"),
                        PathCommand::LineHorizontal(dx) => write!(writer, "h{dx}"),
                        PathCommand::Line(dx, dy) => write!(writer, "l{dx},{dy}"),
                    }?
                }
            } else if !segment.is_open() {
                write!(writer, "z")?;
            }
            write!(writer, r##"" stroke-width="1" stroke="#000"/>"##)?;
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
