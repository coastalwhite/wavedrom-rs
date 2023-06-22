//! All the render options

use crate::color::Color;

macro_rules! replace_default {
    ($property_type:ty) => {
        <$property_type>::default()
    };
    ($_:ty, $default_value:expr) => {
        $default_value
    };
}

#[cfg(feature = "skins")]
macro_rules! replace_ty {
    ($x:ty) => {
        $x
    };
    ($_:ty, $x:ty) => {
        $x
    };
}

#[cfg(feature = "skins")]
macro_rules! replace_merge {
    ($name:expr, $value:expr) => {
        $name = $value
    };
    ($name:expr, $value:expr, $__:ty) => {
        $name.merge_in($value)
    };
}

macro_rules! define_options {
    ( $(#[$struct_doc:meta])* $([$copy:meta])? $struct_name:ident, $(#[$opt_struct_doc:meta])* $opt_struct_name:ident { $( $(#[$property_doc:meta])* $property_name:ident: $property_type:ty$([$opt_property_type:ty])? $(=> $property_default_value:expr)?),+ $(,)? } ) => (
        #[derive(Debug, Clone)]
        $(
        #[$struct_doc]
        )*
        pub struct $struct_name {
            $(
            $(
            #[$property_doc]
            )*
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
        $(
        #[$opt_struct_doc]
        )*
        pub struct $opt_struct_name {
            $(
            $(
            #[$property_doc]
            )*
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
        impl From<$struct_name> for $opt_struct_name {
            fn from(value: $struct_name) -> Self {
                Self {
                $(
                    $property_name: Some(value.$property_name.into()),
                )+
                }
            }
        }

        #[cfg(feature = "skins")]
        impl $struct_name {
            /// Merge a partial configuration into a full configuration
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
    /// The options used while rendering a figure
    RenderOptions,

    /// A subset of the [`RenderOptions`]
    PartialRenderOptions {
        /// The figure background
        background: Option<Color> => Some(Color::WHITE),
        /// The figure's paddings
        padding: FigurePadding[PartialFigurePadding],
        /// The figure's spacings
        spacing: FigureSpacing[PartialFigureSpacing],
        /// The figure's header options
        header: HeaderOptions[PartialHeaderOptions],
        /// The figure's footer options
        footer: FooterOptions[PartialFooterOptions],
        /// The signal options
        signal: SignalOptions[PartialSignalOptions],
        /// The group indicator
        group_indicator: GroupIndicatorOptions[PartialGroupIndicatorOptions],
        /// The arrow / edge options
        edge: EdgeOptions[PartialEdgeOptions],
    }
}

define_options! {
    /// The paddings of the figure
    FigurePadding,

    /// A subset of [`FigurePadding`]
    PartialFigurePadding {
        /// The padding at the top of the figure
        figure_top: u32 => 8,
        /// The padding at the bottom of the figure
        figure_bottom: u32 => 8,
        /// The padding at the left of the figure
        figure_left: u32 => 8,
        /// The padding at the right of the figure
        figure_right: u32 => 8,

        /// The padding at the top of the signal schema
        schema_top: u32 => 8,
        /// The padding at the bottom of the signal schema
        schema_bottom: u32 => 8,
    }
}

define_options! {
    /// The spacings for the figure
    FigureSpacing,

    /// A subset of [`FigureSpacing`]
    PartialFigureSpacing {
        /// The spacing between the signal names and the signal schema
        textbox_to_schema: u32 => 8,
        /// The spacing group indicators and the signal names
        groupbox_to_textbox: u32 => 8,
        /// The between signal lines
        line_to_line: u32 => 8,
    }
}

define_options! {
    /// The header options for the figure
    HeaderOptions,

    /// A subset of [`HeaderOptions`]
    PartialHeaderOptions {
        /// The header font size
        font_size: u32 => 24,
        /// The header height
        height: u32 => 32,
        /// The header text color
        color: Color => Color::BLACK,

        /// The cycle enumeration marker height
        cycle_marker_height: u32 => 12,
        /// The cycle enumeration marker font size
        cycle_marker_fontsize: u32 => 12,
        /// The cycle enumeration marker text color
        cycle_marker_color: Color => Color::BLACK,
    }
}

define_options! {
    /// The footer options for the figure
    FooterOptions,

    /// A subset of [`FooterOptions`]
    PartialFooterOptions {
        /// The footer font size
        font_size: u32 => 24,
        /// The footer height
        height: u32 => 32,
        /// The footer text color
        color: Color => Color::BLACK,

        /// The cycle enumeration marker height
        cycle_marker_height: u32 => 12,
        /// The cycle enumeration marker font size
        cycle_marker_fontsize: u32 => 12,
        /// The cycle enumeration marker text color
        cycle_marker_color: Color => Color::BLACK,
    }
}

define_options! {
    /// The signal options for the figure
    SignalOptions,

    /// A subset of [`SignalOptions`]
    PartialSignalOptions {
        /// The font size of the data text marker
        marker_font_size: u32 => 14,
        /// The text color of the data text marker
        marker_color: Color => Color::BLACK,

        /// The font size of the name
        name_font_size: u32 => 14,
        /// The text color of the name
        name_color: Color => Color::BLACK,

        /// The line color of a gap
        gap_color: Color => Color::BLACK,
        /// The background color of a gap
        gap_background_color: Color => Color::WHITE,

        /// The line color of the signal path
        path_color: Color => Color::BLACK,

        /// The line color of the dashed background cycle hint line
        hint_line_color: Color => Color { red: 0xCC, green: 0xCC, blue: 0xCC },

        /// The line color of the undefined background pattern
        undefined_color: Color => Color::BLACK,
        /// The background color of the undefined background pattern
        undefined_background: Option<Color> => None,

        /// The background colors for the Box2 to Box9 states
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
    /// The group indicator options for the figure
    GroupIndicatorOptions,

    /// A subset of the [`GroupIndicatorOptions`]
    PartialGroupIndicatorOptions {
        /// The width of the group indicator
        width: u32 => 4,
        /// The spacing between group indicators
        spacing: u32 => 4,
        /// The color of the group indicators
        color: Color => Color::BLACK,

        /// The spacing between group indicator labels
        label_spacing: u32 => 4,
        /// The font size of group indicator labels
        label_fontsize: u32 => 14,
        /// The color of group indicator labels
        label_color: Color => Color::BLACK,
    }
}

define_options! {
    /// The arrow / edge options for the figure
    EdgeOptions,

    /// A subset of the [`EdgeOptions`]
    PartialEdgeOptions {
        /// The font size for a node label
        node_font_size: u32 => 14,
        /// The text color for a node label
        node_text_color: Color => Color::BLACK,
        /// The background color for a node label
        node_background_color: Color => Color::WHITE,

        /// The font size for an edge label
        edge_text_font_size: u32 => 14,
        /// The text color for an edge label
        edge_text_color: Color => Color::BLACK,
        /// The background color for an edge label
        edge_text_background_color: Color => Color::WHITE,

        /// The line color for an edge
        edge_color: Color => Color { red: 0, green: 0, blue: 255 },
        /// The arrow color for an edge
        edge_arrow_color: Color => Color { red: 0, green: 0, blue: 255 },
        /// The arrow size for an edge
        edge_arrow_size: u32 => 8,
    }
}

impl GroupIndicatorOptions {
    /// The label spacing added to the label font size
    pub fn label_height(&self) -> u32 {
        self.label_spacing + self.label_fontsize
    }
}


define_options! {
    /// The options that are used during assembly of a
    /// [`SignalFigure`][crate::signal::SignalFigure] or [`SignalPath`].
    #[derive(Copy)]
    PathAssembleOptions,

    /// A subset of the [`PathAssembleOptions`]
    #[derive(Copy)]
    PartialPathAssembleOptions {
        /// The height of a single signal bar
        signal_height: u16 => 24,
        /// The width of a single cycle
        cycle_width: u16 => 48,
        /// The offset from the cycle transition point where a state transition can start
        transition_offset: u16 => 4,
    }
}
