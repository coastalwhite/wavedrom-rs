//! Module with a WaveDrom skin
use serde::{Deserialize, Serialize};

use crate::signal::options::{
    PartialPathAssembleOptions, PartialRenderOptions, PathAssembleOptions, RenderOptions,
};

/// The definition for a WaveDrom skin.
///
/// This is a JSON file that defines options for how to assemble and render a WaveDrom figure.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Skin {
    /// Assemble options given as an optional [`PathAssembleOptions`]
    pub assemble: Option<PartialPathAssembleOptions>,
    /// Render options given as an optional subset of a [`RenderOptions`]
    pub render: Option<PartialRenderOptions>,
}

impl Skin {
    /// Generate a set of options from the [`Skin`].
    ///
    /// If some options was not specified by the skin it is set to the default value.
    pub fn options(self) -> (PathAssembleOptions, RenderOptions) {
        (
            self.assemble
                .map_or_else(PathAssembleOptions::default, PathAssembleOptions::from),
            self.render
                .map_or_else(RenderOptions::default, RenderOptions::from),
        )
    }

    /// Parse a [`Skin`] from a human-friendly / JSON5 file.
    #[cfg(feature = "json5")]
    #[inline]
    pub fn from_json5(s: &str) -> Result<Self, json5::Error> {
        json5::from_str(s)
    }

    /// Parse a [`Skin`] from a JSON file.
    #[cfg(feature = "serde_json")]
    #[inline]
    pub fn from_json(s: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(s)
    }
}
