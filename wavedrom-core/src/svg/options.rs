use crate::color::Color;
use crate::path::SignalOptions;

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub font_size: u32,
    /// TODO: Make this a proper color
    pub background: Option<Color>,
    pub paddings: FigurePadding,
    pub spacings: FigureSpacing,
    pub header: HeaderOptions,
    pub footer: FooterOptions,
    pub wave_dimensions: SignalOptions,
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
    pub width: u32,

    pub spacing: u32,

    pub label_spacing: u32,
    pub label_fontsize: u32,
}

#[derive(Debug, Clone)]
pub struct HeaderOptions {
    pub font_size: u32,
    pub height: u32,

    pub cycle_marker_height: u32,
    pub cycle_marker_fontsize: u32,
}

#[derive(Debug, Clone)]
pub struct FooterOptions {
    pub font_size: u32,
    pub height: u32,

    pub cycle_marker_height: u32,
    pub cycle_marker_fontsize: u32,
}

impl Default for HeaderOptions {
    fn default() -> Self {
        Self {
            font_size: 24,
            height: 32,
            cycle_marker_height: 12,
            cycle_marker_fontsize: 12,
        }
    }
}

impl Default for FooterOptions {
    fn default() -> Self {
        Self {
            font_size: 24,
            height: 32,
            cycle_marker_height: 12,
            cycle_marker_fontsize: 12,
        }
    }
}

impl Default for GroupIndicatorDimension {
    fn default() -> Self {
        Self {
            width: 4,
            spacing: 4,

            label_spacing: 4,
            label_fontsize: 14,
        }
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 14,
            background: None,
            paddings: FigurePadding::default(),
            spacings: FigureSpacing::default(),
            header: HeaderOptions::default(),
            footer: FooterOptions::default(),
            wave_dimensions: SignalOptions::default(),
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
    pub fn label_height(&self) -> u32 {
        self.label_spacing + self.label_fontsize
    }
}

