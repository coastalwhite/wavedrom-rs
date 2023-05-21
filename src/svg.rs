use std::io;

use super::RenderedFigure;
use super::path::{WavePath, WaveDimension};

pub trait ToSvg {
    fn write_svg(&self, io_writer: &mut impl io::Write) -> io::Result<()>;
}

struct SvgWavePath<'a> {
    dimensions: &'a WaveDimension,
    wave_path: &'a WavePath,
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
        for i in 0..=self.num_cycles {
            write!(
                writer,
                r##"<use transform="translate({x})" xlink:href="#cl" />"##,
                x = f64::from(i) * self.wave_dimensions().cycle_width_f64(),
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
                        0.0
                    } else {
                        self.wave_dimensions().wave_height_f64() * f64::from(i)
                            + self.spacings().line_to_line * f64::from(i)
                    }
            )?;

            write!(
                writer,
                r##"<text dominant-baseline="middle" font-family="{font_family}pt" y="{y}pt" font-size="{font_size}px" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                font_family = self.font_family,
                font_size = self.options.font_size,
                y = self.wave_dimensions().wave_height_f64() / 2.0,
                text = line.text,
            )?;

            write!(writer, r##"<g transform="translate({schema_x})">"##,)?;
            SvgWavePath {
                dimensions: self.wave_dimensions(),
                wave_path: &line.path,
            }.write_svg(writer)?;
            write!(writer, r##"</g>"##)?;

            write!(writer, r##"</g>"##)?;
        }

        write!(writer, "</g></svg>")?;

        Ok(())
    }
}

impl<'a> ToSvg for SvgWavePath<'a> { 
    fn write_svg(
        &self,
        writer: &mut impl io::Write,
    ) -> io::Result<()> {
        for (path, container_number) in self.wave_path.to_paths(self.dimensions).into_iter() {
            let fill = match container_number {
                Some(0) => "#ff4040",
                Some(1) => "#5499C7",
                Some(2) => "#58D68D",
                Some(3) => "#A569BD",
                _ => "none",
            };
            write!(writer, r##"<path fill="{fill}" d=""##)?;
            for action in path.actions {
                write!(writer, "{action}")?;
            }
            write!(writer, r##"" stroke-width="1" stroke="#000"/>"##)?;
        }

        Ok(())
    }
}
