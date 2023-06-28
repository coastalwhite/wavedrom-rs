use std::io;

use crate::Font;

use super::Register;

fn to_display_num(n: f64) -> f64 {
    (n * 1000.).round() / 1000.
}

impl Register {
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        const WIDTH: f64 = 400.;
        const HEIGHT: f64 = 100.;

        const PAD_LEFT: f64 = 5.;
        const PAD_RIGHT: f64 = 5.;

        const PAD_TOP: f64 = 25.;

        const BAR_WIDTH: f64 = WIDTH - PAD_LEFT - PAD_RIGHT;
        const BAR_HEIGHT: f64 = 20.;

        const BAR_MIDDLE: f64 = (PAD_TOP + PAD_TOP + BAR_HEIGHT) / 2.;

        const NAME_FONTSIZE: f64 = BAR_HEIGHT * 2. / 5.;
        const BITMARKER_FONTSIZE: f64 = 6.;
        const ATTRIBUTE_FONTSIZE: f64 = NAME_FONTSIZE;

        const HINT_INDENT: f64 = BAR_HEIGHT / 10.;

        const SE_MARKER_X_OFFSET: f64 = 2.;
        const SE_MARKER_Y_OFFSET: f64 = 2.;

        const ATTRIBUTE_Y_OFFSET: f64 = 2.;
        const ATTRIBUTE_Y_SPACING: f64 = 4.;

        let font = Font::default();

        let font_family = font
            .get_font_family_name()
            .unwrap_or_else(|| String::from("Helvetica"));

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}" overflow="hidden" width="{figure_width}" height="{figure_height}">"#,
            figure_width = WIDTH,
            figure_height = HEIGHT,
        )?;

        if self.bit_ranges.is_empty() {
            write!(writer, "</svg>")?;
            return Ok(());
        }

        let bit_width = f64::from(self.width);

        write!(
            writer,
            r##"<defs><g id="bm"><path d="M0,0v{HINT_INDENT}m0,{jump}v{HINT_INDENT}" stroke="#000" fill="none"/></g></defs>"##,
            jump = BAR_HEIGHT - 2. * HINT_INDENT,
        )?;

        write!(writer, "<g>")?;

        let mut offset = 0;
        for bit_range in &self.bit_ranges {
            if bit_range.length == 0 {
                continue;
            }

            let offset_start = offset;
            let offset_end = offset + bit_range.length;

            // Draw backgrounds
            if bit_range.variant != 0 {
                let background = match bit_range.variant {
                    1 => "#B55",
                    2 => "#CCC",
                    _ => "#5B5",
                };

                let width = to_display_num(if offset_end == self.width {
                    BAR_WIDTH - f64::from(offset_start) * BAR_WIDTH / bit_width
                } else {
                    f64::from(bit_range.length) * BAR_WIDTH / bit_width
                });

                write!(
                    writer,
                    r##"<path d="M{x},{PAD_TOP}h{width}v{BAR_HEIGHT}h-{width}v-{BAR_HEIGHT}z" stroke="none" fill="{background}"/>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH - f64::from(offset_end) * BAR_WIDTH / bit_width
                    ),
                )?;
            }

            offset = offset_end;
        }

        let mut offset = 0;
        for bit_range in &self.bit_ranges {
            if bit_range.length == 0 {
                continue;
            }

            let offset_start = offset;
            let offset_end = offset + bit_range.length;

            // Draw bit hint markers
            for i in offset_start + 1..offset_end {
                let i = f64::from(i);

                write!(
                    writer,
                    r##"<use transform="translate({x},{y})" xlink:href="#bm"/>"##,
                    x = to_display_num(PAD_LEFT + BAR_WIDTH - i * BAR_WIDTH / bit_width),
                    y = PAD_TOP
                )?;
            }

            let offset_center = f64::from(offset_start + offset_end) / 2.;

            // Draw the field name
            if let Some(name) = &bit_range.name {
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{NAME_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{name}</tspan></text>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH - offset_center * BAR_WIDTH / bit_width
                    ),
                    y = BAR_MIDDLE,
                )?;
            }

            // Draw the start and end markers
            if bit_range.length == 1 {
                // TODO: Better Centering
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{offset_start}</tspan></text>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH - (offset_center * BAR_WIDTH) / bit_width
                    ),
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                )?;
            } else {
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="end" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH
                            - f64::from(offset_start) * BAR_WIDTH / bit_width
                            - SE_MARKER_X_OFFSET
                    ),
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                    text = offset_start,
                )?;
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="start" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH - f64::from(offset_end) * BAR_WIDTH / bit_width
                            + SE_MARKER_X_OFFSET
                    ),
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                    text = offset_end - 1,
                )?;
            }

            for (i, attribute) in bit_range.attributes.iter().enumerate() {
                if attribute.is_empty() {
                    continue;
                }

                let i = i as u32;

                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{ATTRIBUTE_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{offset_start}</tspan></text>"##,
                    x = to_display_num(
                        PAD_LEFT + BAR_WIDTH - (offset_center * BAR_WIDTH) / bit_width
                    ),
                    y = to_display_num(
                        PAD_TOP
                            + BAR_HEIGHT
                            + ATTRIBUTE_Y_OFFSET
                            + (ATTRIBUTE_FONTSIZE + ATTRIBUTE_Y_SPACING) * f64::from(i)
                    ),
                )?;
            }

            offset = offset_end;

            if offset == self.width {
                break;
            }

            // Draw field separation markers
            write!(
                writer,
                r##"<line x1="{x}" y1="{PAD_TOP}" x2="{x}" y2="{bar_bottom}" stroke="#000"/>"##,
                x = to_display_num(
                    PAD_LEFT + BAR_WIDTH - (f64::from(offset) * BAR_WIDTH) / bit_width
                ),
                bar_bottom = PAD_TOP + BAR_HEIGHT,
            )?;
        }
        write!(writer, "</g>")?;

        write!(
            writer,
            r##"<rect x="{PAD_LEFT}" y="{PAD_TOP}" width="{BAR_WIDTH}" height="{BAR_HEIGHT}" stroke="#000" fill="none"/>"##
        )?;

        write!(writer, "</svg>")?;

        Ok(())
    }
}
