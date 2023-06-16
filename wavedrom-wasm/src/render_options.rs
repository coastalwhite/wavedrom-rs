use std::sync::{Mutex, MutexGuard};

use wavedrom::svg::options::RenderOptions;
use wavedrom::Color;

macro_rules! prefix_fn {
    ($property:expr) => {
        $property
    };
    ($property:expr, $_fn:ident) => {
        &$property
    };
}
macro_rules! surround_fn {
    ($property:expr) => {
        $property
    };
    ($property:expr, $fn:ident) => {
        $fn($property)
    };
}

macro_rules! parameters {
    ($($name:ident [$($property:ident$([$prop_idx:literal])?).+] $([$as:ty])? $({$deserialize_fn:ident, $serialize_fn:ident})?),+ $(,)?) => {
        #[repr(u32)]
        enum RenderParameter {
        $(
            $name,
        )+
        }

        impl RenderParameter {
            fn from_u32(n: u32) -> Option<Self> {
                Some(match n {
                    $(
                    x if x == Self::$name as u32 => Self::$name,
                    )+
                    _ => return None,
                })
            }
        }

        #[no_mangle]
        pub extern "C" fn modify_parameter(parameter: u32, value: u32) {
            let mut options =
                unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
                    .lock()
                    .unwrap();
            let Some(parameter) = RenderParameter::from_u32(parameter) else {
                return;
            };

            match parameter {
                $(
                    RenderParameter::$name => options.$($property$([$prop_idx])?).+ = surround_fn!(value $(, $deserialize_fn)?) $(as $as)?,
                )+
            }
        }

        #[no_mangle]
        pub extern "C" fn get_parameter_default(parameter: u32) -> u32 {
            let options = RenderOptions::default();
            let Some(parameter) = RenderParameter::from_u32(parameter) else {
                return 0;
            };

            match parameter {
                $(
                    RenderParameter::$name => {
                        let value = prefix_fn!(options.$($property$([$prop_idx])?).+$(, $serialize_fn)?);
                        surround_fn!(value $(, $serialize_fn)?) $(as $as as u32)?
                    },
                )+
            }
        }
    };
}

parameters![
    Background[background]{parse_opt_color, serialize_opt_color},

    SignalMarkerFontSize[wave_dimensions.marker_font_size],
    SignalMarkerColor[wave_dimensions.marker_color]{parse_color, serialize_color},

    SignalNameFontSize[wave_dimensions.name_font_size],
    SignalNameColor[wave_dimensions.name_color]{parse_color, serialize_color},

    SignalPathColor[wave_dimensions.path_color]{parse_color, serialize_color},

    SignalHintLineColor[wave_dimensions.hint_line_color]{parse_color, serialize_color},

    SignalHeight[wave_dimensions.signal_height][u16],
    CycleWidth[wave_dimensions.cycle_width][u16],
    TransitionOffset[wave_dimensions.transition_offset][u16],
    
    SignalUndefinedColor[wave_dimensions.undefined_color]{parse_color, serialize_color},
    SignalUndefinedBackgroundColor[wave_dimensions.undefined_background]{parse_opt_color, serialize_opt_color},

    BackgroundBox2[wave_dimensions.backgrounds[0]]{parse_color, serialize_color},
    BackgroundBox3[wave_dimensions.backgrounds[1]]{parse_color, serialize_color},
    BackgroundBox4[wave_dimensions.backgrounds[2]]{parse_color, serialize_color},
    BackgroundBox5[wave_dimensions.backgrounds[3]]{parse_color, serialize_color},
    BackgroundBox6[wave_dimensions.backgrounds[4]]{parse_color, serialize_color},
    BackgroundBox7[wave_dimensions.backgrounds[5]]{parse_color, serialize_color},
    BackgroundBox8[wave_dimensions.backgrounds[6]]{parse_color, serialize_color},
    BackgroundBox9[wave_dimensions.backgrounds[7]]{parse_color, serialize_color},

    PaddingFigureTop[paddings.figure_top],
    PaddingFigureBottom[paddings.figure_bottom],
    PaddingFigureLeft[paddings.figure_left],
    PaddingFigureRight[paddings.figure_right],
    PaddingSchemaTop[paddings.schema_top],
    PaddingSchemaBottom[paddings.schema_bottom],

    SpacingTextboxToSchema[spacings.textbox_to_schema],
    SpacingGroupboxToTextbox[spacings.groupbox_to_textbox],
    SpacingLineToLine[spacings.line_to_line],

    GroupIndicatorWidth[group_indicator_dimensions.width],
    GroupIndicatorSpacing[group_indicator_dimensions.spacing],
    GroupIndicatorColor[group_indicator_dimensions.color]{parse_color, serialize_color},

    GroupIndicatorLabelSpacing[group_indicator_dimensions.label_spacing],
    GroupIndicatorLabelFontSize[group_indicator_dimensions.label_fontsize],
    GroupIndicatorLabelColor[group_indicator_dimensions.label_color]{parse_color, serialize_color},

    HeaderFontSize[header.font_size],
    HeaderHeight[header.height],
    HeaderColor[header.color]{parse_color, serialize_color},

    TopCycleMarkerHeight[header.cycle_marker_height],
    TopCycleMarkerFontSize[header.cycle_marker_fontsize],
    TopCycleMarkerColor[header.cycle_marker_color]{parse_color, serialize_color},

    FooterFontSize[footer.font_size],
    FooterHeight[footer.height],
    FooterColor[footer.color]{parse_color, serialize_color},

    BottomCycleMarkerHeight[footer.cycle_marker_height],
    BottomCycleMarkerFontSize[footer.cycle_marker_fontsize],
    BottomCycleMarkerColor[footer.cycle_marker_color]{parse_color, serialize_color},

    EdgeNodeFontSize[edges.node_font_size],
    EdgeNodeTextColor[edges.node_text_color]{ parse_color, serialize_color },
    EdgeNodeBackgroundColor[edges.node_background_color]{ parse_color, serialize_color },

    EdgeTextFontSize[edges.edge_text_font_size],
    EdgeTextColor[edges.edge_text_color]{ parse_color, serialize_color },
    EdgeTextBackgroundColor[edges.edge_text_background_color]{ parse_color, serialize_color },

    EdgeColor[edges.edge_color]{ parse_color, serialize_color },
    EdgeArrowColor[edges.edge_arrow_color]{ parse_color, serialize_color },
    EdgeArrowSize[edges.edge_arrow_size],
];

fn parse_color(value: u32) -> Color {
    Color {
        red: ((value & 0x00FF_0000) >> 16) as u8,
        green: ((value & 0x0000_FF00) >> 8) as u8,
        blue: ((value & 0x0000_00FF) >> 0) as u8,
    }
}
fn parse_opt_color(value: u32) -> Option<Color> {
    if value & 0xFF00_0000 == 0 {
        return None;
    }

    Some(parse_color(value))
}

fn serialize_color(color: &Color) -> u32 {
    return (0xFF << 24)
        | ((color.red as u32) << 16)
        | ((color.green as u32) << 8)
        | (color.blue as u32);
}

fn serialize_opt_color(color: &Option<Color>) -> u32 {
    let Some(color) = color else {
        return 0;
    };

    serialize_color(color)
}

static mut RENDER_OPTIONS: Option<Mutex<RenderOptions>> = None;

pub fn get_options() -> MutexGuard<'static, RenderOptions> {
    unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
        .lock()
        .unwrap()
}
