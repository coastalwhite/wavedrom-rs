use std::sync::{Mutex, MutexGuard};

use wavedrom::svg::RenderOptions;

#[repr(u32)]
enum RenderParameter {
    FontSize,
    Background,

    SignalHeight,
    CycleWidth,
    TransitionOffset,
    MarkerFontSize,
    BackgroundBox2,
    BackgroundBox3,
    BackgroundBox4,
    BackgroundBox5,
    BackgroundBox6,
    BackgroundBox7,
    BackgroundBox8,
    BackgroundBox9,

    PaddingFigureTop,
    PaddingFigureBottom,
    PaddingFigureLeft,
    PaddingFigureRight,
    PaddingSchemaTop,
    PaddingSchemaBottom,

    SpacingTextboxToSchema,
    SpacingGroupboxToTextbox,
    SpacingLineToLine,

    GroupIndicatorWidth,
    GroupIndicatorSpacing,
    GroupIndicatorLabelSpacing,
    GroupIndicatorLabelFontSize,

    HeaderFontSize,
    HeaderHeight,
    TopCycleMarkerHeight,
    TopCycleMarkerFontSize,

    FooterFontSize,
    FooterHeight,
    BottomCycleMarkerHeight,
    BottomCycleMarkerFontSize,
}

impl RenderParameter {
    fn from_u32(n: u32) -> Option<Self> {
        Some(match n {
            0 => Self::FontSize,
            1 => Self::Background,
            2 => Self::SignalHeight,
            3 => Self::CycleWidth,
            4 => Self::TransitionOffset,
            5 => Self::MarkerFontSize,
            6 => Self::BackgroundBox2,
            7 => Self::BackgroundBox3,
            8 => Self::BackgroundBox4,
            9 => Self::BackgroundBox5,
            10 => Self::BackgroundBox6,
            11 => Self::BackgroundBox7,
            12 => Self::BackgroundBox8,
            13 => Self::BackgroundBox9,

            14 => Self::PaddingFigureTop,
            15 => Self::PaddingFigureBottom,
            16 => Self::PaddingFigureLeft,
            17 => Self::PaddingFigureRight,
            18 => Self::PaddingSchemaTop,
            19 => Self::PaddingSchemaBottom,

            20 => Self::SpacingTextboxToSchema,
            21 => Self::SpacingGroupboxToTextbox,
            22 => Self::SpacingLineToLine,

            23 => Self::GroupIndicatorWidth,
            24 => Self::GroupIndicatorSpacing,
            25 => Self::GroupIndicatorLabelSpacing,
            26 => Self::GroupIndicatorLabelFontSize,

            27 => Self::HeaderFontSize,
            28 => Self::HeaderHeight,
            29 => Self::TopCycleMarkerHeight,
            30 => Self::TopCycleMarkerFontSize,

            31 => Self::FooterFontSize,
            32 => Self::FooterHeight,
            33 => Self::BottomCycleMarkerHeight,
            34 => Self::BottomCycleMarkerFontSize,
            _ => return None,
        })
    }
}

fn parse_background(value: u32) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (value & 0x00FF_0000) >> 16,
        (value & 0xFF00) >> 8,
        value & 0xFF
    )
}
fn parse_opt_background(value: u32) -> Option<String> {
    if value & 0xFF00_0000 == 0 {
        return None;
    }

    Some(parse_background(value))
}

fn serialize_background(color: &String) -> u32 {
    if color.len() != 7 {
        return 0;
    }

    let r = &color[1..3];
    let g = &color[3..5];
    let b = &color[5..7];

    let Ok(r) = u32::from_str_radix(r, 16) else {
        return 0;
    };
    let Ok(g) = u32::from_str_radix(g, 16) else {
        return 0;
    };
    let Ok(b) = u32::from_str_radix(b, 16) else {
        return 0;
    };

    return (0xFF << 24) | (r << 16) | (g << 8) | b;
}

fn serialize_opt_background(color: &Option<String>) -> u32 {
    let Some(color) = color else {
        return 0;
    };

    serialize_background(color)
}

static mut RENDER_OPTIONS: Option<Mutex<RenderOptions>> = None;

#[no_mangle]
pub extern "C" fn modify_parameter(parameter: u32, value: u32) {
    let mut options =
        unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
            .lock()
            .unwrap();
    let Some(parameter) = RenderParameter::from_u32(parameter) else {
        return;
    };

    use RenderParameter::*;
    match parameter {
        FontSize => options.font_size = value,
        Background => options.background = parse_opt_background(value),

        SignalHeight => options.wave_dimensions.signal_height = value as u16,
        CycleWidth => options.wave_dimensions.cycle_width = value as u16,
        TransitionOffset => options.wave_dimensions.transition_offset = value as u16,
        MarkerFontSize => options.wave_dimensions.marker_font_size = value,
        BackgroundBox2 => options.wave_dimensions.backgrounds[0] = parse_background(value),
        BackgroundBox3 => options.wave_dimensions.backgrounds[1] = parse_background(value),
        BackgroundBox4 => options.wave_dimensions.backgrounds[2] = parse_background(value),
        BackgroundBox5 => options.wave_dimensions.backgrounds[3] = parse_background(value),
        BackgroundBox6 => options.wave_dimensions.backgrounds[4] = parse_background(value),
        BackgroundBox7 => options.wave_dimensions.backgrounds[5] = parse_background(value),
        BackgroundBox8 => options.wave_dimensions.backgrounds[6] = parse_background(value),
        BackgroundBox9 => options.wave_dimensions.backgrounds[7] = parse_background(value),

        PaddingFigureTop => options.paddings.figure_top = value,
        PaddingFigureBottom => options.paddings.figure_bottom = value,
        PaddingFigureLeft => options.paddings.figure_left = value,
        PaddingFigureRight => options.paddings.figure_right = value,
        PaddingSchemaTop => options.paddings.schema_top = value,
        PaddingSchemaBottom => options.paddings.schema_bottom = value,

        SpacingTextboxToSchema => options.spacings.textbox_to_schema = value,
        SpacingGroupboxToTextbox => options.spacings.groupbox_to_textbox = value,
        SpacingLineToLine => options.spacings.line_to_line = value,

        GroupIndicatorWidth => options.group_indicator_dimensions.width = value,
        GroupIndicatorSpacing => options.group_indicator_dimensions.spacing = value,
        GroupIndicatorLabelSpacing => options.group_indicator_dimensions.label_spacing = value,
        GroupIndicatorLabelFontSize => options.group_indicator_dimensions.label_fontsize = value,

        HeaderFontSize => options.header.font_size = value,
        HeaderHeight => options.header.height = value,
        TopCycleMarkerHeight => options.header.cycle_marker_height = value,
        TopCycleMarkerFontSize => options.header.cycle_marker_font_size = value,

        FooterFontSize => options.footer.font_size = value,
        FooterHeight => options.footer.height = value,
        BottomCycleMarkerHeight => options.footer.cycle_marker_height = value,
        BottomCycleMarkerFontSize => options.footer.cycle_marker_font_size = value,
    }
}

#[no_mangle]
pub extern "C" fn get_parameter_default(parameter: u32) -> u32 {
    let options = RenderOptions::default();
    let Some(parameter) = RenderParameter::from_u32(parameter) else {
        return 0;
    };

    use RenderParameter::*;
    match parameter {
        FontSize => options.font_size,
        Background => serialize_opt_background(&options.background),

        SignalHeight => options.wave_dimensions.signal_height as u32,
        CycleWidth => options.wave_dimensions.cycle_width as u32,
        TransitionOffset => options.wave_dimensions.transition_offset as u32,
        MarkerFontSize => options.wave_dimensions.marker_font_size,
        BackgroundBox2 => serialize_background(&options.wave_dimensions.backgrounds[0]),
        BackgroundBox3 => serialize_background(&options.wave_dimensions.backgrounds[1]),
        BackgroundBox4 => serialize_background(&options.wave_dimensions.backgrounds[2]),
        BackgroundBox5 => serialize_background(&options.wave_dimensions.backgrounds[3]),
        BackgroundBox6 => serialize_background(&options.wave_dimensions.backgrounds[4]),
        BackgroundBox7 => serialize_background(&options.wave_dimensions.backgrounds[5]),
        BackgroundBox8 => serialize_background(&options.wave_dimensions.backgrounds[6]),
        BackgroundBox9 => serialize_background(&options.wave_dimensions.backgrounds[7]),

        PaddingFigureTop => options.paddings.figure_top,
        PaddingFigureBottom => options.paddings.figure_bottom,
        PaddingFigureLeft => options.paddings.figure_left,
        PaddingFigureRight => options.paddings.figure_right,
        PaddingSchemaTop => options.paddings.schema_top,
        PaddingSchemaBottom => options.paddings.schema_bottom,

        SpacingTextboxToSchema => options.spacings.textbox_to_schema,
        SpacingGroupboxToTextbox => options.spacings.groupbox_to_textbox,
        SpacingLineToLine => options.spacings.line_to_line,

        GroupIndicatorWidth => options.group_indicator_dimensions.width,
        GroupIndicatorSpacing => options.group_indicator_dimensions.spacing,
        GroupIndicatorLabelSpacing => options.group_indicator_dimensions.label_spacing,
        GroupIndicatorLabelFontSize => options.group_indicator_dimensions.label_fontsize,

        HeaderFontSize => options.header.font_size,
        HeaderHeight => options.header.height,
        TopCycleMarkerHeight => options.header.cycle_marker_height,
        TopCycleMarkerFontSize => options.header.cycle_marker_font_size,

        FooterFontSize => options.footer.font_size,
        FooterHeight => options.footer.height,
        BottomCycleMarkerHeight => options.footer.cycle_marker_height,
        BottomCycleMarkerFontSize => options.footer.cycle_marker_font_size,
    }
}

pub fn get_options() -> MutexGuard<'static, RenderOptions> {
    unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
        .lock()
        .unwrap()
}
