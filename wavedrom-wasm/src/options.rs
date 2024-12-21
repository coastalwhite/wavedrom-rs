use std::sync::{LazyLock, Mutex};

use wavedrom::skin::Skin;
use wavedrom::Color;
use wavedrom::Options;

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
    (
        $($name:ident [$($property:ident$([$prop_idx:literal])?).+] $([$as:ty])? $({$deserialize_fn:ident, $serialize_fn:ident})?),+ $(,)?
    ) => {
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
            let mut options = OPTIONS.lock().unwrap();

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
        pub extern "C" fn get_parameter(parameter: u32) -> u32 {
            let options = OPTIONS.lock().unwrap();

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
    SignalHeight[signal.path.signal_height][u16],
    CycleWidth[signal.path.cycle_width][u16],
    TransitionOffset[signal.path.transition_offset][u16],

    Background[background]{parse_opt_color, serialize_opt_color},

    SignalMarkerFontSize[signal.marker_font_size],
    SignalMarkerColor[signal.marker_color]{parse_color, serialize_color},

    SignalNameFontSize[signal.name_font_size],
    SignalNameColor[signal.name_color]{parse_color, serialize_color},

    SignalGapColor[signal.gap_color]{parse_color, serialize_color},
    SignalGapBackgroundColor[signal.gap_background_color]{parse_color, serialize_color},

    SignalPathColor[signal.path_color]{parse_color, serialize_color},

    SignalHintLineColor[signal.hint_line_color]{parse_color, serialize_color},

    SignalUndefinedColor[signal.undefined_color]{parse_color, serialize_color},
    SignalUndefinedBackgroundColor[undefined_background]{parse_opt_color, serialize_opt_color},

    BackgroundBox2[backgrounds[0]]{parse_color, serialize_color},
    BackgroundBox3[backgrounds[1]]{parse_color, serialize_color},
    BackgroundBox4[backgrounds[2]]{parse_color, serialize_color},
    BackgroundBox5[backgrounds[3]]{parse_color, serialize_color},
    BackgroundBox6[backgrounds[4]]{parse_color, serialize_color},
    BackgroundBox7[backgrounds[5]]{parse_color, serialize_color},
    BackgroundBox8[backgrounds[6]]{parse_color, serialize_color},
    BackgroundBox9[backgrounds[7]]{parse_color, serialize_color},

    PaddingFigureTop[padding.figure_top],
    PaddingFigureBottom[padding.figure_bottom],
    PaddingFigureLeft[padding.figure_left],
    PaddingFigureRight[padding.figure_right],
    PaddingSchemaTop[padding.schema_top],
    PaddingSchemaBottom[padding.schema_bottom],

    SpacingTextboxToSchema[spacing.textbox_to_schema],
    SpacingGroupboxToTextbox[spacing.groupbox_to_textbox],
    SpacingLineToLine[spacing.line_to_line],

    GroupIndicatorWidth[signal.group_indicator.width],
    GroupIndicatorSpacing[signal.group_indicator.spacing],
    GroupIndicatorColor[signal.group_indicator.color]{parse_color, serialize_color},

    GroupIndicatorLabelSpacing[signal.group_indicator.label_spacing],
    GroupIndicatorLabelFontSize[signal.group_indicator.label_fontsize],
    GroupIndicatorLabelColor[signal.group_indicator.label_color]{parse_color, serialize_color},

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

    EdgeNodeFontSize[signal.edge.node_font_size],
    EdgeNodeTextColor[signal.edge.node_text_color]{ parse_color, serialize_color },
    EdgeNodeBackgroundColor[signal.edge.node_background_color]{ parse_color, serialize_color },

    EdgeTextFontSize[signal.edge.edge_text_font_size],
    EdgeTextColor[signal.edge.edge_text_color]{ parse_color, serialize_color },
    EdgeTextBackgroundColor[signal.edge.edge_text_background_color]{ parse_color, serialize_color },

    EdgeColor[signal.edge.edge_color]{ parse_color, serialize_color },
    EdgeArrowColor[signal.edge.edge_arrow_color]{ parse_color, serialize_color },
    EdgeArrowSize[signal.edge.edge_arrow_size],

    RegisterBarWidth[reg.bar_width],
    RegisterBarHeight[reg.bar_height],

    RegisterHintIndent[reg.hint_indent],

    RegisterNameFontsize[reg.name_fontsize],
    RegisterBitmarkerFontsize[reg.bit_marker_fontsize],
    RegisterAttributeFontsize[reg.attribute_fontsize],

    RegisterPaddingTop[reg.padding.top],
    RegisterPaddingBottom[reg.padding.bottom],
    RegisterPaddingLeft[reg.padding.left],
    RegisterPaddingRight[reg.padding.right],

    RegisterSpacingLane[reg.spacing.lane_spacing],
    RegisterSpacingAttribute[reg.spacing.attribute_spacing],

    RegisterOffsetBitmarkerX[reg.offset.bit_marker_x],
    RegisterOffsetBitmarkerY[reg.offset.bit_marker_y],
    RegisterOffsetAttributeY[reg.offset.attribute_y],
];

fn parse_color(value: u32) -> Color {
    Color {
        red: ((value & 0x00FF_0000) >> 16) as u8,
        green: ((value & 0x0000_FF00) >> 8) as u8,
        blue: (value & 0x0000_00FF) as u8,
    }
}
fn parse_opt_color(value: u32) -> Option<Color> {
    if value & 0xFF00_0000 == 0 {
        return None;
    }

    Some(parse_color(value))
}

fn serialize_color(color: &Color) -> u32 {
    (0xFF << 24) | ((color.red as u32) << 16) | ((color.green as u32) << 8) | (color.blue as u32)
}

fn serialize_opt_color(color: &Option<Color>) -> u32 {
    let Some(color) = color else {
        return 0;
    };

    serialize_color(color)
}

pub static OPTIONS: LazyLock<Mutex<Options>> = LazyLock::new(|| Mutex::new(Options::default()));

#[inline]
pub fn merge_in_skin_internal(json: &str) -> Result<(), ()> {
    let Ok(skin) = Skin::from_json5(json) else {
        return Err(());
    };

    OPTIONS.lock().unwrap().merge_in(skin.0);
    Ok(())
}

#[inline]
pub fn reset() {
    *OPTIONS.lock().unwrap() = Options::default();
}

#[inline]
pub fn export() -> wavedrom::json5::Result<String> {
    let skin = Skin(OPTIONS.lock().unwrap().clone().into());
    wavedrom::json5::to_string(&skin)
}
