//! All the render options

use crate::color::Color;

define_options! {
    /// The options used while rendering a figure
    SignalOptions,

    /// A subset of the [`RenderOptions`]
    PartialSignalOptions {
        /// The group indicator
        group_indicator: GroupIndicatorOptions[PartialGroupIndicatorOptions],
        /// The arrow / edge options
        edge: EdgeOptions[PartialEdgeOptions],
        /// The path options
        path: PathOptions[PartialPathOptions],

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
    PathOptions,

    /// A subset of the [`PathAssembleOptions`]
    #[derive(Copy)]
    PartialPathOptions {
        /// The height of a single signal bar
        signal_height: u16 => 24,
        /// The width of a single cycle
        cycle_width: u16 => 48,
        /// The offset from the cycle transition point where a state transition can start
        transition_offset: u16 => 4,
    }
}
