use std::io;

use crate::Font;

use super::{FieldString, Lane, LaneBitRange, RegisterFigure};

fn to_display_num(n: f64) -> f64 {
    (n * 1000.).round() / 1000.
}

const WIDTH: f64 = 800.;

const PAD_LEFT: f64 = 4.;
const PAD_RIGHT: f64 = 4.;

const LANE_SPACING: f64 = 4.;

const PAD_TOP: f64 = 4.;
const PAD_BOTTOM: f64 = 4.;

const BAR_Y: f64 = SE_MARKER_Y_OFFSET + BITMARKER_FONTSIZE;

const BAR_WIDTH: f64 = WIDTH - PAD_LEFT - PAD_RIGHT;
const BAR_HEIGHT: f64 = 40.;

const BAR_MIDDLE: f64 = (BAR_Y + BAR_Y + BAR_HEIGHT) / 2.;

const NAME_FONTSIZE: f64 = BAR_HEIGHT * 2. / 5.;
const BITMARKER_FONTSIZE: f64 = 12.;
const ATTRIBUTE_FONTSIZE: f64 = NAME_FONTSIZE;

const HINT_INDENT: f64 = BAR_HEIGHT / 10.;

const SE_MARKER_X_OFFSET: f64 = 2.;
const SE_MARKER_Y_OFFSET: f64 = 2.;

const ATTRIBUTE_Y_OFFSET: f64 = 4.;
const ATTRIBUTE_Y_SPACING: f64 = 4.;

impl RegisterFigure {
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        let mut height = PAD_TOP + PAD_BOTTOM;
        let mut displayed_lanes = 0;
        for lane in &self.lanes {
            if lane.is_empty() {
                height += LANE_SPACING;

                continue;
            }

            displayed_lanes += 1;
            height += lane.display_height() + LANE_SPACING;
        }

        let height = if displayed_lanes == 0 {
            0.
        } else {
            height - LANE_SPACING
        };

        write!(
            writer,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewport="0 0 {figure_width} {figure_height}" overflow="hidden" width="{figure_width}" height="{figure_height}">"#,
            figure_width = WIDTH,
            figure_height = to_display_num(height),
        )?;

        if self.lanes.is_empty() {
            write!(writer, "</svg>")?;
            return Ok(());
        }

        write!(
            writer,
            r##"<defs><g id="bm"><path d="M0,0v{HINT_INDENT}m0,{jump}v{HINT_INDENT}" stroke="#000" fill="none"/></g></defs>"##,
            jump = BAR_HEIGHT - 2. * HINT_INDENT,
        )?;

        let mut y = PAD_TOP;

        for lane in &self.lanes {
            if lane.is_empty() {
                y += LANE_SPACING;

                continue;
            }

            write!(writer, r##"<g transform="translate({PAD_LEFT},{y})">"##)?;

            lane.write_svg(writer)?;

            let lane_height = lane.display_height();

            y += lane_height + LANE_SPACING;

            write!(writer, "</g>")?;
        }

        write!(writer, "</svg>")?;
        Ok(())
    }
}

impl Lane {
    pub fn write_svg(&self, writer: &mut impl io::Write) -> io::Result<()> {
        if self.width == 0 {
            return Ok(());
        }

        let mut offset = 0;
        for bit_range in &self.bit_ranges {
            if bit_range.length == 0 {
                continue;
            }

            let offset_end = offset + bit_range.length;

            bit_range.write_svg(writer, offset, self.width, self.start_bit)?;

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
                r##"<path d="M{x},{BAR_Y}v{BAR_HEIGHT}" stroke="#000" stroke-width="2"/>"##,
                x = to_display_num(
                    BAR_WIDTH - (f64::from(offset) * BAR_WIDTH) / f64::from(self.width)
                ),
            )?;
        }

        write!(
            writer,
            r##"<path d="M0,{BAR_Y}h{BAR_WIDTH}v{BAR_HEIGHT}H0V{BAR_Y}z" stroke="#000" stroke-width="2" fill="none"/>"##
        )?;

        Ok(())
    }

    pub fn display_height(&self) -> f64 {
        let bit_marker_height = BITMARKER_FONTSIZE + SE_MARKER_Y_OFFSET;
        let bar_height = BAR_HEIGHT;
        let max_attributes = self
            .bit_ranges
            .iter()
            .map(|bit_range| bit_range.attributes.len() as u32)
            .max()
            .unwrap_or_default();
        let attributes_height = if max_attributes == 0 {
            0.
        } else {
            let max_attributes = f64::from(max_attributes);
            max_attributes * ATTRIBUTE_FONTSIZE
                + (max_attributes * ATTRIBUTE_Y_SPACING)
                + ATTRIBUTE_Y_OFFSET
        };

        bit_marker_height + bar_height + attributes_height
    }
}

impl LaneBitRange {
    pub fn write_svg(
        &self,
        writer: &mut impl io::Write,
        offset: u32,
        bit_width: u32,
        start_bit: u32,
    ) -> io::Result<()> {
        let font = Font::default();

        let font_family = font
            .get_font_family_name()
            .unwrap_or_else(|| String::from("Helvetica"));

        if self.length == 0 {
            return Ok(());
        }

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
                BAR_WIDTH - f64::from(offset_start) * BAR_WIDTH / f64::from(bit_width)
            } else {
                f64::from(self.length) * BAR_WIDTH / f64::from(bit_width)
            });

            write!(
                writer,
                r##"<path d="M{x},{BAR_Y}h{width}v{BAR_HEIGHT}h-{width}v-{BAR_HEIGHT}z" stroke="none" fill="{background}"/>"##,
                x = to_display_num(
                    BAR_WIDTH - f64::from(offset_end) * BAR_WIDTH / f64::from(bit_width)
                ),
            )?;
        }

        // Draw bit hint markers
        for i in offset_start + 1..offset_end {
            let i = f64::from(i);

            write!(
                writer,
                r##"<use x="{x}" y="{y}" xlink:href="#bm"/>"##,
                x = to_display_num(BAR_WIDTH - i * BAR_WIDTH / f64::from(bit_width)),
                y = BAR_Y,
            )?;
        }

        let offset_center = f64::from(offset_start + offset_end) / 2.;

        // Draw the field name
        if let Some(name) = &self.name {
            match name {
                FieldString::Text(name) => {
                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{NAME_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{name}</tspan></text>"##,
                        x = to_display_num(
                            BAR_WIDTH - offset_center * BAR_WIDTH / f64::from(bit_width)
                        ),
                        y = BAR_MIDDLE,
                    )?;
                }
                FieldString::Binary(mut binary) => {
                    write!(
                        writer,
                        r##"<text y="{y}" text-anchor="middle" dominant-baseline="middle" font-family="{font_family}" font-size="{NAME_FONTSIZE}" fill="#000" letter-spacing="0">"##,
                        y = BAR_MIDDLE,
                    )?;
                    for i in 0..self.length {
                        write!(
                            writer,
                            r#"<tspan x="{x}">{bit}</tspan>"#,
                            x = to_display_num(
                                BAR_WIDTH - (f64::from(offset_end - i - 1) + 0.5) * BAR_WIDTH / f64::from(bit_width)
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
                r##"<text x="{x}" y="{BITMARKER_FONTSIZE}" text-anchor="middle" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{bit_idx}</tspan></text>"##,
                x = to_display_num(BAR_WIDTH - (offset_center * BAR_WIDTH) / f64::from(bit_width)),
                bit_idx = start_bit + offset_start,
            )?;
        } else {
            write!(
                writer,
                r##"<text x="{x}" y="{BITMARKER_FONTSIZE}" text-anchor="end" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = to_display_num(
                    BAR_WIDTH
                        - f64::from(offset_start) * BAR_WIDTH / f64::from(bit_width)
                        - SE_MARKER_X_OFFSET
                ),
                text = start_bit + offset_start,
            )?;
            write!(
                writer,
                r##"<text x="{x}" y="{BITMARKER_FONTSIZE}" text-anchor="start" font-family="{font_family}" font-size="{BITMARKER_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{text}</tspan></text>"##,
                x = to_display_num(
                    BAR_WIDTH - f64::from(offset_end) * BAR_WIDTH / f64::from(bit_width)
                        + SE_MARKER_X_OFFSET
                ),
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

                    write!(
                        writer,
                        r##"<text x="{x}" y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{ATTRIBUTE_FONTSIZE}" fill="#000" letter-spacing="0"><tspan>{offset_start}</tspan></text>"##,
                        x = to_display_num(
                            BAR_WIDTH - (offset_center * BAR_WIDTH) / f64::from(bit_width)
                        ),
                        y = to_display_num(
                            BAR_Y
                                + BAR_HEIGHT
                                + ATTRIBUTE_Y_OFFSET
                                + (ATTRIBUTE_FONTSIZE + ATTRIBUTE_Y_SPACING) * f64::from(i)
                        ),
                    )?;
                }
                FieldString::Binary(mut binary) => {
                    write!(
                        writer,
                        r##"<text y="{y}" text-anchor="middle" dominant-baseline="hanging" font-family="{font_family}" font-size="{NAME_FONTSIZE}" fill="#000" letter-spacing="0">"##,
                        y = to_display_num(
                            BAR_Y
                                + BAR_HEIGHT
                                + ATTRIBUTE_Y_OFFSET
                                + (ATTRIBUTE_FONTSIZE + ATTRIBUTE_Y_SPACING) * f64::from(i)
                        ),
                    )?;
                    for j in 0..self.length {
                        write!(
                            writer,
                            r#"<tspan x="{x}">{bit}</tspan>"#,
                            x = to_display_num(
                                BAR_WIDTH
                                    - (f64::from(offset_end - j - 1) + 0.5) * BAR_WIDTH
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
