use std::io;

use crate::escape::escape_str;
use crate::{Font, Options};

use super::{FieldString, Lane, LaneBitRange, RegisterFigure};

const DISPLAY_PRECISION: u8 = 3;
const DISPLAY_PRECISION_ROUNDING: f64 = {
    let mut n = 10.;
    let mut i = 0;
    loop {
        if i >= DISPLAY_PRECISION {
            break n;
        }

        n *= 10.;
        i += 1;
    }
};

fn to_display_num(n: f64) -> f64 {
    (n * DISPLAY_PRECISION_ROUNDING).round() / DISPLAY_PRECISION_ROUNDING
}

impl RegisterFigure {
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        self.write_svg_with_options(writer, &Options::default())
    }

    pub fn write_svg_with_options(
        &self,
        writer: &mut impl io::Write,
        options: &Options,
    ) -> io::Result<()> {
        let mut height = f64::from(options.reg.padding.top + options.reg.padding.bottom);
        let mut displayed_lanes = 0;
        for lane in &self.lanes {
            if lane.is_empty() {
                height += f64::from(options.reg.spacing.lane_spacing);

                continue;
            }

            displayed_lanes += 1;
            height += lane.display_height(options, self.vspace)
                + f64::from(options.reg.spacing.lane_spacing);
        }

        let height = if displayed_lanes == 0 {
            0.
        } else {
            height - f64::from(options.reg.spacing.lane_spacing)
        };

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}" overflow="hidden" width="{figure_width}" height="{figure_height}">"#,
            figure_width =
                options.reg.padding.left + options.reg.padding.right + options.reg.bar_width,
            figure_height = to_display_num(height),
        )?;

        if self.lanes.is_empty() {
            write!(writer, "</svg>")?;
            return Ok(());
        }

        write!(
            writer,
            r##"<defs><g id="bm"><path d="M0,0v{hint_indent}m0,{jump}v{hint_indent}" stroke="#000" fill="none"/></g></defs>"##,
            jump = options.reg.bar_height - 2 * options.reg.hint_indent,
            hint_indent = options.reg.hint_indent,
        )?;

        let mut y = f64::from(options.reg.padding.top);

        for lane in &self.lanes {
            if lane.is_empty() {
                y += f64::from(options.reg.spacing.lane_spacing);

                continue;
            }

            write!(
                writer,
                r##"<g transform="translate({x},{y})">"##,
                x = options.reg.padding.left,
            )?;

            lane.write_svg(writer, options)?;

            let lane_height = lane.display_height(options, self.vspace);

            y += lane_height + f64::from(options.reg.spacing.lane_spacing);

            write!(writer, "</g>")?;
        }

        write!(writer, "</svg>")?;
        Ok(())
    }
}

impl Lane {
    fn write_svg(&self, writer: &mut impl io::Write, options: &Options) -> io::Result<()> {
        if self.width == 0 {
            return Ok(());
        }

        let bar_width = options.reg.bar_width;
        let bar_height = options.reg.bar_height;
        let bar_y = options.reg.bit_marker_fontsize + options.reg.offset.bit_marker_y;

        let mut offset = 0;
        for bit_range in &self.bit_ranges {
            if bit_range.length == 0 {
                continue;
            }

            let offset_end = offset + bit_range.length;

            bit_range.write_svg(writer, offset, self.width, self.start_bit, options)?;

            offset = offset_end;
        }

        let amount_of_field_bits: u32 = self
            .bit_ranges
            .iter()
            .map(|bit_range| bit_range.length)
            .sum();

        if amount_of_field_bits != self.width {
            LaneBitRange::new_padding(self.width - amount_of_field_bits).write_svg(
                writer,
                amount_of_field_bits,
                self.width,
                self.start_bit,
                options,
            )?;
        }

        let mut offset = 0;
        for bit_range in &self.bit_ranges {
            if bit_range.length == 0 {
                continue;
            }

            offset = offset + bit_range.length;

            if offset == self.width {
                break;
            }

            // Draw field separation markers
            write!(
                writer,
                r##"<path d="M{x},{bar_y}v{bar_height}" stroke="#000" stroke-width="2"/>"##,
                x = to_display_num(
                    f64::from(bar_width) - (f64::from(offset * bar_width)) / f64::from(self.width)
                ),
            )?;
        }

        write!(
            writer,
            r##"<path d="M0,{bar_y}h{bar_width}v{bar_height}H0V{bar_y}z" stroke="#000" stroke-width="2" fill="none"/>"##
        )?;

        Ok(())
    }

    pub fn display_height(&self, options: &Options, vspace: Option<u32>) -> f64 {
        let bit_marker_height = options.reg.bit_marker_fontsize + options.reg.offset.bit_marker_y;
        let bar_height = vspace.unwrap_or(options.reg.bar_height);
        let max_attributes = self
            .bit_ranges
            .iter()
            .map(|bit_range| bit_range.attributes.len() as u32)
            .max()
            .unwrap_or_default();
        let attributes_height = if max_attributes == 0 {
            0
        } else {
            max_attributes * options.reg.attribute_fontsize
                + ((max_attributes - 1) * options.reg.spacing.attribute_spacing)
                + options.reg.offset.attribute_y
        };

        f64::from(bit_marker_height + bar_height + attributes_height)
    }
}

impl LaneBitRange {
    fn write_svg(
        &self,
        writer: &mut impl io::Write,
        offset: u32,
        bit_width: u32,
        start_bit: u32,
        options: &Options,
    ) -> io::Result<()> {
        if self.length == 0 {
            return Ok(());
        }

        let font = Font::default();

        let font_family = font
            .get_font_family_name()
            .unwrap_or_else(|| String::from("Helvetica"));

        let bar_width = options.reg.bar_width;
        let bar_height = options.reg.bar_height;
        let bar_y = options.reg.bit_marker_fontsize + options.reg.offset.bit_marker_y;

        let bar_middle = f64::from(bar_y + bar_y + bar_height) / 2.;

        let offset_start = offset;
        let offset_end = offset + self.length;

        // Draw background
        if self.variant != 0 {
            let background = match self.variant {
                1 => "#B55",
                2 => "#CCC",
                _ => "#5B5",
            };

            // TODO: Can this be deleted
            let width = to_display_num(if offset_end == bit_width {
                f64::from(bar_width) - f64::from(offset_start * bar_width) / f64::from(bit_width)
            } else {
                f64::from(self.length * bar_width) / f64::from(bit_width)
            });

            write!(
                writer,
                r##"<path d="M{x},{bar_y}h{width}v{bar_height}h-{width}v-{bar_height}z" stroke="none" fill="{background}"/>"##,
                x = to_display_num(
                    f64::from(bar_width) - f64::from(offset_end * bar_width) / f64::from(bit_width)
                ),
            )?;
        }

        // Draw bit hint markers
        for i in offset_start + 1..offset_end {
            let i = f64::from(i);

            write!(
                writer,
                r##"<use x="{x}" y="{bar_y}" xlink:href="#bm"/>"##,
                x = to_display_num(
                    f64::from(bar_width) - i * f64::from(bar_width) / f64::from(bit_width)
                ),
            )?;
        }

        let offset_center = f64::from(offset_start + offset_end) / 2.;

        // Draw the field name
        if let Some(name) = &self.name {
            match name {
                FieldString::Text(name) => {
                    let name = escape_str(name);
                    write!(
                        writer,
                        r##"<text x="{x}" y="{bar_middle}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0"><tspan>{name}</tspan></text>"##,
                        x = to_display_num(
                            f64::from(bar_width)
                                - offset_center * f64::from(bar_width) / f64::from(bit_width)
                        ),
                        fontsize = options.reg.name_fontsize,
                    )?;
                }
                FieldString::Binary(mut binary) => {
                    write!(
                        writer,
                        r##"<text y="{bar_middle}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0">"##,
                        fontsize = options.reg.name_fontsize,
                    )?;
                    for i in 0..self.length {
                        write!(
                            writer,
                            r#"<tspan x="{x}">{bit}</tspan>"#,
                            x = to_display_num(
                                f64::from(bar_width)
                                    - (f64::from(offset_end - i - 1) + 0.5) * f64::from(bar_width)
                                        / f64::from(bit_width)
                            ),
                            bit = binary & 1,
                        )?;

                        binary &= !1;
                        binary >>= 1;
                    }
                    write!(writer, "</text>")?;
                }
            }
        }

        // Draw the start and end markers
        if self.length == 1 {
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="middle" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0"><tspan>{bit_idx}</tspan></text>"##,
                x = to_display_num(
                    f64::from(bar_width)
                        - (offset_center * f64::from(bar_width)) / f64::from(bit_width)
                ),
                y = options.reg.bit_marker_fontsize,
                fontsize = options.reg.bit_marker_fontsize,
                bit_idx = start_bit + offset_start,
            )?;
        } else {
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="end" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = to_display_num(
                    f64::from(bar_width)
                        - f64::from(offset_start) * f64::from(bar_width) / f64::from(bit_width)
                        - f64::from(options.reg.offset.bit_marker_x)
                ),
                y = options.reg.bit_marker_fontsize,
                fontsize = options.reg.bit_marker_fontsize,
                text = start_bit + offset_start,
            )?;
            write!(
                writer,
                r##"<text x="{x}" y="{y}" text-anchor="start" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = to_display_num(
                    f64::from(bar_width)
                        - f64::from(offset_end) * f64::from(bar_width) / f64::from(bit_width)
                        + f64::from(options.reg.offset.bit_marker_x)
                ),
                y = options.reg.bit_marker_fontsize,
                fontsize = options.reg.bit_marker_fontsize,
                text = start_bit + offset_end - 1,
            )?;
        }

        for (i, attribute) in self.attributes.iter().enumerate() {
            let i = i as u32;

            match attribute {
                FieldString::Text(attribute) => {
                    if attribute.is_empty() {
                        continue;
                    }

                    let attribute = escape_str(attribute);

                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0"><tspan>{attribute}</tspan></text>"##,
                        x = to_display_num(
                            f64::from(bar_width)
                                - (offset_center * f64::from(bar_width)) / f64::from(bit_width)
                        ),
                        y = bar_y
                            + bar_height
                            + options.reg.offset.attribute_y
                            + (options.reg.attribute_fontsize
                                + options.reg.spacing.attribute_spacing)
                                * i,
                        fontsize = options.reg.attribute_fontsize,
                    )?;
                }
                FieldString::Binary(mut binary) => {
                    write!(
                        writer,
                        r##"<text y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{fontsize}" fill="#000" letter-spacing="0">"##,
                        y = bar_y
                            + bar_height
                            + options.reg.offset.attribute_y
                            + (options.reg.attribute_fontsize
                                + options.reg.spacing.attribute_spacing)
                                * i,
                        fontsize = options.reg.attribute_fontsize,
                    )?;
                    for j in 0..self.length {
                        write!(
                            writer,
                            r#"<tspan x="{x}">{bit}</tspan>"#,
                            x = to_display_num(
                                f64::from(bar_width)
                                    - (f64::from(offset_end - j - 1) + 0.5) * f64::from(bar_width)
                                        / f64::from(bit_width)
                            ),
                            bit = binary & 1,
                        )?;

                        binary &= !1;
                        binary >>= 1;
                    }
                    write!(writer, "</text>")?;
                }
            }
        }

        Ok(())
    }
}
