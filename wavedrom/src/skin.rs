//! Module with a WaveDrom skin
use serde::{Deserialize, Serialize};

use crate::{Options, PartialOptions};

/// The definition for a WaveDrom skin.
///
/// This is a JSON file that defines options for how to assemble and render a WaveDrom figure.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Skin(pub PartialOptions);

impl Skin {
    /// Generate a set of options from the [`Skin`].
    ///
    /// If some options was not specified by the skin it is set to the default value.
    pub fn options(self) -> Options {
        Options::from(self.0)
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
