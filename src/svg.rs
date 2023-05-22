use std::io;

use crate::path::PathCommand;

use super::path::RenderedWavePath;
use super::RenderedFigure;

pub trait ToSvg {
    fn write_svg(&self, io_writer: &mut impl io::Write) -> io::Result<()>;
}

impl<'a> ToSvg for RenderedFigure<'a> {
    fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {width} {height}">"#,
            width = self.width(),
            height = self.height(),
        )?;

        write!(
            writer,
            r##"<defs><g id="cl"><path fill="none" d="M0,0v{height}" stroke-width="1" stroke-dasharray="2" stroke="#CCC" /></g></defs>"##,
            height = self.schema_height,
        )?;

        write!(
            writer,
            r##"<g transform="translate({padding_x},{padding_y})">"##,
            padding_x = self.paddings().figure_left,
            padding_y = self.paddings().figure_top,
        )?;

        let schema_x = self.textbox_width + self.spacings().textbox_to_schema;

        write!(writer, r##"<g transform="translate({schema_x})">"##,)?;
        for i in 0..=u64::from(self.num_cycles) {
            write!(
                writer,
                r##"<use transform="translate({x})" xlink:href="#cl" />"##,
                x = i * u64::from(self.wave_dimensions().cycle_width)
            )?;
        }
        write!(writer, r##"</g>"##)?;

        for (i, line) in self.lines.iter().enumerate() {
            let Ok(i) = u32::try_from(i) else {
                break;
            };

            write!(
                writer,
                r##"<g transform="translate(0,{y})">"##,
                y = self.paddings().schema_top
                    + if i == 0 {
                        0
                    } else {
                        u32::from(self.wave_dimensions().wave_height) * i
                            + self.spacings().line_to_line * i
                    }
            )?;

            write!(
                writer,
                r##"<text dominant-baseline="middle" font-family="{font_family}pt" y="{y}pt" font-size="{font_size}px" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                font_family = self.font_family,
                font_size = self.options.font_size,
                y = self.wave_dimensions().wave_height / 2,
                text = line.text,
            )?;

            write!(writer, r##"<g transform="translate({schema_x})">"##,)?;
            line.path
                .render_with_options(&self.options.wave_dimensions)
                .write_svg(writer)?;
            write!(writer, r##"</g>"##)?;

            write!(writer, r##"</g>"##)?;
        }

        write!(writer, "</g></svg>")?;

        Ok(())
    }
}

impl ToSvg for RenderedWavePath {
    fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
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

            write!(writer, r##"<path fill="{fill}" d=""##)?;
            for action in segment.actions() {
                write!(writer, "{action}")?;
            }

            // If there is a `no_stroke` element, we need to divide up the filling and the
            // stroking.
            if !segment.is_fully_stroked() {
                write!(writer, r##"" stroke="none"/>"##)?;

                write!(writer, r##"<path fill="none" d=""##)?;
                for action in segment.actions() {
                    match action {
                        PathCommand::LineVerticalNoStroke(dy) => write!(writer, "m0,{dy}")?,
                        PathCommand::Close => {}
                        _ => write!(writer, "{action}")?,
                    }
                }
            }
            write!(writer, r##"" stroke-width="1" stroke="#000"/>"##)?;
        }

        Ok(())
    }
}
