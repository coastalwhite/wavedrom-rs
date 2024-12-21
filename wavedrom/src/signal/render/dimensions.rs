use crate::signal::options::SignalOptions;
use crate::signal::AssembledFigure;
use crate::{Font, Options};

pub struct SvgDimensions<'a> {
    figure: &'a AssembledFigure<'a>,
    options: &'a Options,
    textbox_width: Option<u32>,
}

impl<'a> SvgDimensions<'a> {
    pub fn new(figure: &'a AssembledFigure<'a>, font: Font, options: &'a Options) -> Self {
        let has_textbox = !figure.lines.iter().all(|line| line.text.is_empty());
        let textbox_width = has_textbox.then(|| {
            figure
                .lines
                .iter()
                .map(|line| font.get_text_width(line.text, options.signal.name_font_size))
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
        let Options { spacing, .. } = self.options;

        let mut width = self.schema_width();

        if self.has_grouping() {
            width += self.grouping_width() + spacing.groupbox_to_textbox;
        }

        if self.has_textbox() {
            width += self.textbox_width() + spacing.textbox_to_schema;
        }

        width
    }

    #[inline]
    pub fn inner_x(&self) -> u32 {
        self.options.padding.figure_left
    }

    #[inline]
    pub fn figure_width(&self) -> u32 {
        let Options { padding, .. } = self.options;
        padding.figure_left + padding.figure_right + self.inner_width()
    }

    #[inline]
    pub fn figure_height(&self) -> u32 {
        let Options { padding, .. } = self.options;

        padding.figure_top
            + self.header_height()
            + self.schema_height()
            + self.footer_height()
            + padding.figure_bottom
    }

    #[inline]
    pub fn header_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    pub fn header_height(&self) -> u32 {
        let Options { header, .. } = self.options;

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
        self.options.padding.figure_left
    }

    #[inline]
    pub fn header_y(&self) -> u32 {
        self.options.padding.figure_top
    }

    #[inline]
    pub fn footer_width(&self) -> u32 {
        self.inner_width()
    }

    #[inline]
    pub fn footer_height(&self) -> u32 {
        let Options { footer, .. } = self.options;

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
            + self.options.padding.schema_top
            + if idx == 0 {
                0
            } else {
                (u32::from(self.options.signal.path.signal_height)
                    + self.options.spacing.line_to_line)
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
            x += self.grouping_width() + self.options.spacing.groupbox_to_textbox;
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

        let SignalOptions {
            group_indicator, ..
        } = &self.options.signal;

        let sum_indicator_widths = max_group_depth * group_indicator.width;
        let spacing = (max_group_depth - 1) * group_indicator.spacing;
        let num_labels = self
            .figure
            .group_label_at_depth
            .iter()
            .filter(|x| **x)
            .count() as u32;
        let label_widths = if num_labels == 0 {
            0
        } else {
            num_labels * group_indicator.label_height() - group_indicator.label_spacing
        };

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
            x += self.textbox_width() + self.options.spacing.textbox_to_schema;
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
        if self.figure.lines.is_empty() {
            return 0;
        }

        let Options {
            padding, spacing, ..
        } = self.options;

        let num_lines = self.num_lines();

        padding.schema_top
            + padding.schema_bottom
            + spacing.line_to_line * (num_lines - 1)
            + self.wave_height() * num_lines
    }

    #[inline]
    pub fn cycle_width(&self) -> u32 {
        (self.figure.hscale * self.options.signal.path.cycle_width).into()
    }

    #[inline]
    pub fn wave_height(&self) -> u32 {
        self.options.signal.path.signal_height.into()
    }

    #[inline]
    pub fn num_lines(&self) -> u32 {
        self.figure.lines.len() as u32
    }
}
