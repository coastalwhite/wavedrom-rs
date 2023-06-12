use std::sync::{Mutex, MutexGuard};

use wavedrom::svg::options::RenderOptions;

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
    ($($idx:literal, $name:ident [$($property:ident$([$prop_idx:literal])?).+] $([$as:ty])? $(, $deserialize_fn:ident, $serialize_fn:ident)?),+ $(,)?) => {
        enum RenderParameter {
        $(
            $name,
        )+
        }
        
        impl RenderParameter {
            fn from_u32(n: u32) -> Option<Self> {
                Some(match n {
                    $(
                    $idx => Self::$name,
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
    0, FontSize [font_size],
    1, Background [background], parse_opt_background, serialize_opt_background,

    2, SignalHeight [wave_dimensions.signal_height] [u16],
    3, CycleWidth [wave_dimensions.cycle_width] [u16],
    4, TransitionOffset [wave_dimensions.transition_offset] [u16],
    5, MarkerFontSize [wave_dimensions.marker_font_size],
    6, BackgroundBox2 [wave_dimensions.backgrounds[0]], parse_background, serialize_background,
    7, BackgroundBox3 [wave_dimensions.backgrounds[1]], parse_background, serialize_background,
    8, BackgroundBox4 [wave_dimensions.backgrounds[2]], parse_background, serialize_background,
    9, BackgroundBox5 [wave_dimensions.backgrounds[3]], parse_background, serialize_background,
    10, BackgroundBox6 [wave_dimensions.backgrounds[4]], parse_background, serialize_background,
    11, BackgroundBox7 [wave_dimensions.backgrounds[5]], parse_background, serialize_background,
    12, BackgroundBox8 [wave_dimensions.backgrounds[6]], parse_background, serialize_background,
    13, BackgroundBox9 [wave_dimensions.backgrounds[7]], parse_background, serialize_background,

    14, PaddingFigureTop [paddings.figure_top],
    15, PaddingFigureBottom [paddings.figure_bottom],
    16, PaddingFigureLeft [paddings.figure_left],
    17, PaddingFigureRight [paddings.figure_right],
    18, PaddingSchemaTop [paddings.schema_top],
    19, PaddingSchemaBottom [paddings.schema_bottom],

    20, SpacingTextboxToSchema [spacings.textbox_to_schema],
    21, SpacingGroupboxToTextbox [spacings.groupbox_to_textbox],
    22, SpacingLineToLine [spacings.line_to_line],

    23, GroupIndicatorWidth [group_indicator_dimensions.width],
    24, GroupIndicatorSpacing [group_indicator_dimensions.spacing],
    25, GroupIndicatorLabelSpacing [group_indicator_dimensions.label_spacing],
    26, GroupIndicatorLabelFontSize [group_indicator_dimensions.label_fontsize],

    27, HeaderFontSize [header.font_size],
    28, HeaderHeight [header.height],
    29, TopCycleMarkerHeight [header.cycle_marker_height],
    30, TopCycleMarkerFontSize [header.cycle_marker_font_size],

    31, FooterFontSize [footer.font_size],
    32, FooterHeight [footer.height],
    33, BottomCycleMarkerHeight [footer.cycle_marker_height],
    34, BottomCycleMarkerFontSize [footer.cycle_marker_font_size],
];

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

pub fn get_options() -> MutexGuard<'static, RenderOptions> {
    unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
        .lock()
        .unwrap()
}
