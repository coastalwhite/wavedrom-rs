define_options! {
    /// The options used while rendering a register figure
    RegisterRenderOptions,

    /// A subset of the [`RegisterRenderOptions`]
    PartialRegisterRenderOptions {
        padding: RegisterPaddings[PartialRegisterPaddings],
        spacings: RegisterSpacings[PartialRegisterSpacings],
        offsets: RegisterOffsets[PartialRegisterOffsets],

        bar_width: u32 => 800,
        bar_height: u32 => 40,

        hint_indent: u32 => 4,

        name_fontsize: u32 => 16,
        bit_marker_fontsize: u32 => 12,
        attribute_fontsize: u32 => 16,
    }
}

define_options! {
    /// The options used while rendering a register figure
    RegisterPaddings,

    /// A subset of the [`RegisterRenderOptions`]
    PartialRegisterPaddings {
        top: u32 => 4,
        bottom: u32 => 4,
        left: u32 => 4,
        right: u32 => 4,
    }
}

define_options! {
    /// The options used while rendering a register figure
    RegisterSpacings,

    /// A subset of the [`RegisterRenderOptions`]
    PartialRegisterSpacings {
        lane_spacing: u32 => 4,
        attribute_spacing: u32 => 4,
    }
}

define_options! {
    /// The options used while rendering a register figure
    RegisterOffsets,

    /// A subset of the [`RegisterRenderOptions`]
    PartialRegisterOffsets {
        bit_marker_x_offset: u32 => 2,
        bit_marker_y_offset: u32 => 2,

        attribute_y_offset: u32 => 4,
    }
}
