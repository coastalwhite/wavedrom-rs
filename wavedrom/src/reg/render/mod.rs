use std::io;

use crate::Font;

use super::Register;

impl Register {
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 100;

        const PAD_LEFT: u32 = 5;
        const PAD_RIGHT: u32 = 5;

        const PAD_TOP: u32 = 25;

        const BAR_WIDTH: u32 = WIDTH - PAD_LEFT - PAD_RIGHT;
        const BAR_HEIGHT: u32 = 20;

        const NAME_FONTSIZE: u32 = BAR_HEIGHT * 2 / 5;
        const BITMARKER_FONTSIZE: u32 = 6;
        const ATTRIBUTE_FONTSIZE: u32 = NAME_FONTSIZE;

        const HINT_INDENT: u32 = BAR_HEIGHT / 10;

        const SE_MARKER_X_OFFSET: u32 = 2;
        const SE_MARKER_Y_OFFSET: u32 = 2;

        const ATTRIBUTE_Y_OFFSET: u32 = 2;
        const ATTRIBUTE_Y_SPACING: u32 = 4;

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

        write!(
            writer,
            r##"<defs><g id="bm"><path d="M0,0v{HINT_INDENT}m0,{jump}v{HINT_INDENT}" stroke="#000" fill="none"/></g></defs>"##,
            jump = BAR_HEIGHT - 2 * HINT_INDENT,
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

                write!(
                    writer,
                    r##"<path d="M{x},{PAD_TOP}h{width}v{BAR_HEIGHT}h-{width}v-{BAR_HEIGHT}z" stroke="none" fill="{background}"/>"##,
                    x = PAD_LEFT + offset_start * BAR_WIDTH / self.width,
                    width = bit_range.length * BAR_WIDTH / self.width,
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

            let offset_center = (offset_start + offset_end) / 2;

            // Draw bit hint markers
            for i in offset + 1..offset + bit_range.length {
                write!(
                    writer,
                    r##"<use transform="translate({x},{y})" xlink:href="#bm"/>"##,
                    x = PAD_LEFT + i * BAR_WIDTH / self.width,
                    y = PAD_TOP
                )?;
            }

            // Draw the field name
            if let Some(name) = &bit_range.name {
                let needs_half = (bit_range.length & 1) == 1;

                // TODO: Better Centering
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{NAME_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{name}</tspan></text>"##,
                    x = PAD_LEFT
                        + (offset_center * BAR_WIDTH + if needs_half { BAR_WIDTH / 2 } else { 0 })
                            / self.width,
                    y = (PAD_TOP + PAD_TOP + BAR_HEIGHT) / 2,
                )?;
            }

            // Draw the start and end markers
            if bit_range.length == 1 {
                let needs_half = (bit_range.length & 1) == 1;

                // TODO: Better Centering
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{offset_start}</tspan></text>"##,
                    x = PAD_LEFT
                        + (offset_center * BAR_WIDTH + if needs_half { BAR_WIDTH / 2 } else { 0 })
                            / self.width,
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                )?;
            } else {
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="start" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                    x = PAD_LEFT + offset_start * BAR_WIDTH / self.width + SE_MARKER_X_OFFSET,
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                    text = offset_start,
                )?;
                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="end" dominant-baseline="text-bottom" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                    x = PAD_LEFT + offset_end * BAR_WIDTH / self.width - SE_MARKER_X_OFFSET,
                    y = PAD_TOP - SE_MARKER_Y_OFFSET,
                    text = offset_end - 1,
                )?;
            }

            for (i, attribute) in bit_range.attributes.iter().enumerate() {
                if attribute.is_empty() {
                    continue;
                }

                let i = i as u32;

                let needs_half = (bit_range.length & 1) == 1;

                write!(
                    writer,
                    r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{ATTRIBUTE_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{offset_start}</tspan></text>"##,
                    x = PAD_LEFT
                        + (offset_center * BAR_WIDTH + if needs_half { BAR_WIDTH / 2 } else { 0 })
                            / self.width,
                    y = PAD_TOP
                        + BAR_HEIGHT
                        + ATTRIBUTE_Y_OFFSET
                        + (ATTRIBUTE_FONTSIZE + ATTRIBUTE_Y_SPACING) * i,
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
                x = PAD_LEFT + (offset * BAR_WIDTH) / self.width,
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
