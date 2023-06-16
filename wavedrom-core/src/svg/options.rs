use crate::color::Color;

macro_rules! replace_default {
    ($property_type:ty) => {
        <$property_type>::default()
    };
    ($_:ty, $default_value:expr) => {
        $default_value
    };
}

macro_rules! replace_ty {
    ($x:ty) => {
        $x
    };
    ($_:ty, $x:ty) => {
        $x
    };
}

macro_rules! replace_merge {
    ($name:expr, $value:expr) => {
        $name = $value
    };
    ($name:expr, $value:expr, $__:ty) => {
        $name.merge_in($value)
    };
}

macro_rules! define_options {
    ( $struct_name:ident, $opt_struct_name:ident { $( $property_name:ident: $property_type:ty$([$opt_property_type:ty])? $(=> $property_default_value:expr)?),+ $(,)? } ) => (
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            $(
            pub $property_name: $property_type,
            )+
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $(
                    $property_name: replace_default!($property_type$(, $property_default_value)?),
                    )+
                }
            }
        }

        #[cfg(feature = "skins")]
        #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
        pub struct $opt_struct_name {
            $(
            pub $property_name: Option<replace_ty!($property_type$(, $opt_property_type)?)>,
            )+
        }

        #[cfg(feature = "skins")]
        impl From<$opt_struct_name> for $struct_name {
            fn from(opt: $opt_struct_name) -> Self {
                Self {
                $(
                    $property_name: opt.$property_name.map_or_else(
                        ||replace_default!($property_type$(, $property_default_value)?),
                        |v| v.into(),
                    ),
                )+
                }
            }
        }

        #[cfg(feature = "skins")]
        impl $struct_name {
            pub fn merge_in(&mut self, opt: $opt_struct_name) {
                $(
                if let Some(value) = opt.$property_name {
                    replace_merge!(self.$property_name, value$(, $opt_property_type)?);
                }
                )+
            }
        }
    )
}

define_options! {
    RenderOptions, PartialRenderOptions {
        background: Option<Color> => None,
        padding: FigurePadding[PartialFigurePadding],
        spacing: FigureSpacing[PartialFigureSpacing],
        header: HeaderOptions[PartialHeaderOptions],
        footer: FooterOptions[PartialFooterOptions],
        signal: SignalOptions[PartialSignalOptions],
        group_indicator: GroupIndicatorOptions[PartialGroupIndicatorOptions],
        edge: EdgeOptions[PartialEdgeOptions],
    }
}

define_options! {
    FigurePadding, PartialFigurePadding {
        figure_top: u32 => 8,
        figure_bottom: u32 => 8,
        figure_left: u32 => 8,
        figure_right: u32 => 8,

        schema_top: u32 => 8,
        schema_bottom: u32 => 8,
    }
}

define_options! {
    FigureSpacing, PartialFigureSpacing {
        textbox_to_schema: u32 => 8,
        groupbox_to_textbox: u32 => 8,
        line_to_line: u32 => 8,
    }
}

define_options! {
    HeaderOptions, PartialHeaderOptions {
        font_size: u32 => 24,
        height: u32 => 32,
        color: Color => Color::BLACK,

        cycle_marker_height: u32 => 12,
        cycle_marker_fontsize: u32 => 12,
        cycle_marker_color: Color => Color::BLACK,
    }
}

define_options! {
    FooterOptions, PartialFooterOptions {
        font_size: u32 => 24,
        height: u32 => 32,
        color: Color => Color::BLACK,

        cycle_marker_height: u32 => 12,
        cycle_marker_fontsize: u32 => 12,
        cycle_marker_color: Color => Color::BLACK,
    }
}

define_options! {
    SignalOptions, PartialSignalOptions {
        marker_font_size: u32 => 14,
        marker_color: Color => Color::BLACK,

        name_font_size: u32 => 14,
        name_color: Color => Color::BLACK,

        gap_color: Color => Color::BLACK,
        gap_background_color: Color => Color::WHITE,

        path_color: Color => Color::BLACK,

        hint_line_color: Color => Color { red: 0xCC, green: 0xCC, blue: 0xCC },

        undefined_color: Color => Color::BLACK,
        undefined_background: Option<Color> => None,

        backgrounds: [Color; 8] => [
                Color { red: 0xFF, green: 0xFF, blue: 0xFF },
                Color { red: 0xF7, green: 0xF7, blue: 0xA1 },
                Color { red: 0xF9, green: 0xD4, blue: 0x9F },
                Color { red: 0xAD, green: 0xDE, blue: 0xFF },
                Color { red: 0xAC, green: 0xD5, blue: 0xB6 },
                Color { red: 0xA4, green: 0xAB, blue: 0xE1 },
                Color { red: 0xE8, green: 0xA8, blue: 0xF0 },
                Color { red: 0xFB, green: 0xDA, blue: 0xDA },
        ],
    }
}

define_options! {
    GroupIndicatorOptions, PartialGroupIndicatorOptions {
        width: u32 => 4,
        spacing: u32 => 4,
        color: Color => Color::BLACK,

        label_spacing: u32 => 4,
        label_fontsize: u32 => 14,
        label_color: Color => Color::BLACK,
    }
}

define_options! {
    EdgeOptions, PartialEdgeOptions {
        node_font_size: u32 => 14,
        node_text_color: Color => Color::BLACK,
        node_background_color: Color => Color::WHITE,

        edge_text_font_size: u32 => 14,
        edge_text_color: Color => Color::BLACK,
        edge_text_background_color: Color => Color::WHITE,

        edge_color: Color => Color { red: 0, green: 0, blue: 255 },
        edge_arrow_color: Color => Color { red: 0, green: 0, blue: 255 },
        edge_arrow_size: u32 => 8,
    }
}

impl GroupIndicatorOptions {
    pub fn label_height(&self) -> u32 {
        self.label_spacing + self.label_fontsize
    }
}
