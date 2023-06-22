use std::sync::Mutex;

use wavedrom::signal::options::{PathAssembleOptions, RenderOptions};
use wavedrom::skin::Skin;
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
    ([$assemble:ident, $render:ident], $($name:ident [$($property:ident$([$prop_idx:literal])?).+] $([$as:ty])? $({$deserialize_fn:ident, $serialize_fn:ident})?),+ $(,)?) => {
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
            let mut $render =
                unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
                    .lock()
                    .unwrap();
            let mut $assemble =
                unsafe { ASSEMBLE_OPTIONS.get_or_insert_with(|| Mutex::new(PathAssembleOptions::default())) }
                    .lock()
                    .unwrap();
            let Some(parameter) = RenderParameter::from_u32(parameter) else {
                return;
            };

            match parameter {
                $(
                    RenderParameter::$name => $($property$([$prop_idx])?).+ = surround_fn!(value $(, $deserialize_fn)?) $(as $as)?,
                )+
            }
        }

        #[no_mangle]
        pub extern "C" fn get_parameter(parameter: u32) -> u32 {
            let $assemble = unsafe { ASSEMBLE_OPTIONS.get_or_insert_with(|| Mutex::new(PathAssembleOptions::default())) }
                    .lock()
                    .unwrap();
            let $render = unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
                    .lock()
                    .unwrap();
            let Some(parameter) = RenderParameter::from_u32(parameter) else {
                return 0;
            };

            match parameter {
                $(
                    RenderParameter::$name => {
                        let value = prefix_fn!($($property$([$prop_idx])?).+$(, $serialize_fn)?);
                        surround_fn!(value $(, $serialize_fn)?) $(as $as as u32)?
                    },
                )+
            }
        }
    };
}

parameters![
    [assemble, render],

    SignalHeight[assemble.signal_height][u16],
    CycleWidth[assemble.cycle_width][u16],
    TransitionOffset[assemble.transition_offset][u16],

    Background[render.background]{parse_opt_color, serialize_opt_color},

    SignalMarkerFontSize[render.signal.marker_font_size],
    SignalMarkerColor[render.signal.marker_color]{parse_color, serialize_color},

    SignalNameFontSize[render.signal.name_font_size],
    SignalNameColor[render.signal.name_color]{parse_color, serialize_color},

    SignalGapColor[render.signal.gap_color]{parse_color, serialize_color},
    SignalGapBackgroundColor[render.signal.gap_background_color]{parse_color, serialize_color},

    SignalPathColor[render.signal.path_color]{parse_color, serialize_color},

    SignalHintLineColor[render.signal.hint_line_color]{parse_color, serialize_color},

    SignalUndefinedColor[render.signal.undefined_color]{parse_color, serialize_color},
    SignalUndefinedBackgroundColor[render.signal.undefined_background]{parse_opt_color, serialize_opt_color},

    BackgroundBox2[render.signal.backgrounds[0]]{parse_color, serialize_color},
    BackgroundBox3[render.signal.backgrounds[1]]{parse_color, serialize_color},
    BackgroundBox4[render.signal.backgrounds[2]]{parse_color, serialize_color},
    BackgroundBox5[render.signal.backgrounds[3]]{parse_color, serialize_color},
    BackgroundBox6[render.signal.backgrounds[4]]{parse_color, serialize_color},
    BackgroundBox7[render.signal.backgrounds[5]]{parse_color, serialize_color},
    BackgroundBox8[render.signal.backgrounds[6]]{parse_color, serialize_color},
    BackgroundBox9[render.signal.backgrounds[7]]{parse_color, serialize_color},

    PaddingFigureTop[render.padding.figure_top],
    PaddingFigureBottom[render.padding.figure_bottom],
    PaddingFigureLeft[render.padding.figure_left],
    PaddingFigureRight[render.padding.figure_right],
    PaddingSchemaTop[render.padding.schema_top],
    PaddingSchemaBottom[render.padding.schema_bottom],

    SpacingTextboxToSchema[render.spacing.textbox_to_schema],
    SpacingGroupboxToTextbox[render.spacing.groupbox_to_textbox],
    SpacingLineToLine[render.spacing.line_to_line],

    GroupIndicatorWidth[render.group_indicator.width],
    GroupIndicatorSpacing[render.group_indicator.spacing],
    GroupIndicatorColor[render.group_indicator.color]{parse_color, serialize_color},

    GroupIndicatorLabelSpacing[render.group_indicator.label_spacing],
    GroupIndicatorLabelFontSize[render.group_indicator.label_fontsize],
    GroupIndicatorLabelColor[render.group_indicator.label_color]{parse_color, serialize_color},

    HeaderFontSize[render.header.font_size],
    HeaderHeight[render.header.height],
    HeaderColor[render.header.color]{parse_color, serialize_color},

    TopCycleMarkerHeight[render.header.cycle_marker_height],
    TopCycleMarkerFontSize[render.header.cycle_marker_fontsize],
    TopCycleMarkerColor[render.header.cycle_marker_color]{parse_color, serialize_color},

    FooterFontSize[render.footer.font_size],
    FooterHeight[render.footer.height],
    FooterColor[render.footer.color]{parse_color, serialize_color},

    BottomCycleMarkerHeight[render.footer.cycle_marker_height],
    BottomCycleMarkerFontSize[render.footer.cycle_marker_fontsize],
    BottomCycleMarkerColor[render.footer.cycle_marker_color]{parse_color, serialize_color},

    EdgeNodeFontSize[render.edge.node_font_size],
    EdgeNodeTextColor[render.edge.node_text_color]{ parse_color, serialize_color },
    EdgeNodeBackgroundColor[render.edge.node_background_color]{ parse_color, serialize_color },

    EdgeTextFontSize[render.edge.edge_text_font_size],
    EdgeTextColor[render.edge.edge_text_color]{ parse_color, serialize_color },
    EdgeTextBackgroundColor[render.edge.edge_text_background_color]{ parse_color, serialize_color },

    EdgeColor[render.edge.edge_color]{ parse_color, serialize_color },
    EdgeArrowColor[render.edge.edge_arrow_color]{ parse_color, serialize_color },
    EdgeArrowSize[render.edge.edge_arrow_size],
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

static mut ASSEMBLE_OPTIONS: Option<Mutex<PathAssembleOptions>> = None;
static mut RENDER_OPTIONS: Option<Mutex<RenderOptions>> = None;

pub fn get_assemble_options() -> &'static PathAssembleOptions {
    unsafe { ASSEMBLE_OPTIONS.get_or_insert_with(|| Mutex::new(PathAssembleOptions::default())) }
        .get_mut()
        .unwrap()
}

pub fn get_render_options() -> &'static RenderOptions {
    unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
        .get_mut()
        .unwrap()
}

#[inline]
pub fn merge_in_skin_internal(json: &str) -> Result<(), ()> {
    let Ok(skin) = Skin::from_json5(json) else {
        return Err(());
    };

    if let Some(assemble) = skin.assemble {
        unsafe {
            ASSEMBLE_OPTIONS.get_or_insert_with(|| Mutex::new(PathAssembleOptions::default()))
        }
        .get_mut()
        .unwrap()
        .merge_in(assemble);
    }

    if let Some(render) = skin.render {
        unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
            .get_mut()
            .unwrap()
            .merge_in(render);
    }

    Ok(())
}

#[inline]
pub fn reset() {
    *unsafe {
        ASSEMBLE_OPTIONS.get_or_insert_with(|| Mutex::new(PathAssembleOptions::default()))
    }
    .get_mut()
    .unwrap() = PathAssembleOptions::default();

    *unsafe { RENDER_OPTIONS.get_or_insert_with(|| Mutex::new(RenderOptions::default())) }
        .get_mut()
        .unwrap() = RenderOptions::default();
}

#[inline]
pub fn export() -> wavedrom::json5::Result<String> {
    let assemble = *get_assemble_options();
    let render = get_render_options().clone();

    let skin = Skin {
        assemble: Some(assemble.into()),
        render: Some(render.into()),
    };

    wavedrom::json5::to_string(&skin)
}
