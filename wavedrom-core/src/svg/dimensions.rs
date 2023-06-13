use crate::svg::font::Font;
use crate::svg::options::RenderOptions;
use crate::AssembledFigure;

pub struct SvgDimensions<'a> {
    figure: &'a AssembledFigure<'a>,
    options: &'a RenderOptions,
    textbox_width: Option<u32>,
}

impl<'a> SvgDimensions<'a> {
    pub fn new(figure: &'a AssembledFigure<'a>, font: Font, options: &'a RenderOptions) -> Self {
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

    pub fn inner_width(&self) -> u32 {
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
    pub fn inner_x(&self) -> u32 {
        self.options.paddings.figure_left
    }

    #[inline]
    pub fn figure_width(&self) -> u32 {
        let RenderOptions { paddings, .. } = self.options;
        paddings.figure_left + paddings.figure_right + self.inner_width()
    }

    #[inline]
    pub fn figure_height(&self) -> u32 {
        let RenderOptions { paddings, .. } = self.options;

        paddings.figure_top
            + self.header_height()
            + self.schema_height()
            + self.footer_height()
            + paddings.figure_bottom
    }

    #[inline]
    pub fn header_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    pub fn header_height(&self) -> u32 {
        let RenderOptions { header, .. } = self.options;

        let mut height = 0;

        if self.figure.header_text.is_some() {
            height += header.height;
        }

        if self.figure.top_cycle_marker.is_some() {
            height += header.cycle_marker_height;
        }

        height
    }

    #[inline]
    pub fn header_x(&self) -> u32 {
        self.options.paddings.figure_left
    }

    #[inline]
    pub fn header_y(&self) -> u32 {
        self.options.paddings.figure_top
    }

    #[inline]
    pub fn footer_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    pub fn footer_height(&self) -> u32 {
        let RenderOptions { footer, .. } = self.options;

        let mut height = 0;

        if self.figure.footer_text.is_some() {
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
    pub fn footer_y(&self) -> u32 {
        self.schema_y() + self.schema_height()
    }

    pub fn has_textbox(&self) -> bool {
        self.figure.lines.iter().any(|line| !line.text.is_empty())
    }
    #[inline]
    pub fn textbox_width(&self) -> u32 {
        self.textbox_width.unwrap_or(0)
    }

    pub fn signal_top(&self, idx: u32) -> u32 {
        self.schema_y()
            + self.options.paddings.schema_top
            + if idx == 0 {
                0
            } else {
                (u32::from(self.options.wave_dimensions.signal_height)
                    + self.options.spacings.line_to_line)
                    * idx
            }
    }

    // #[inline]
    // fn textbox_height(&self) -> u32 {
    //     self.schema_height()
    // }

    #[inline]
    pub fn textbox_x(&self) -> u32 {
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
    pub fn has_grouping(&self) -> bool {
        self.figure.max_group_depth != 0
    }

    #[inline]
    pub fn grouping_x(&self) -> u32 {
        self.inner_x()
    }

    // #[inline]
    // fn grouping_y(&self) -> u32 {
    //     self.header_y() + self.header_height()
    // }

    pub fn grouping_width(&self) -> u32 {
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
    pub fn schema_x(&self) -> u32 {
        let mut x = self.textbox_x();

        if self.has_textbox() {
            x += self.textbox_width() + self.options.spacings.textbox_to_schema;
        }

        x
    }

    #[inline]
    pub fn schema_y(&self) -> u32 {
        self.header_y() + self.header_height()
    }

    #[inline]
    pub fn schema_width(&self) -> u32 {
        self.figure.num_cycles * self.cycle_width()
    }

    pub fn schema_height(&self) -> u32 {
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
    pub fn cycle_width(&self) -> u32 {
        (self.figure.hscale * self.options.wave_dimensions.cycle_width).into()
    }

    #[inline]
    pub fn wave_height(&self) -> u32 {
        self.options.wave_dimensions.signal_height.into()
    }

    #[inline]
    pub fn num_lines(&self) -> u32 {
        self.figure.lines.len() as u32
    }
}
