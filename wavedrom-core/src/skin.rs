use serde::{Deserialize, Serialize};

use crate::svg::options::{PartialRenderOptions, RenderOptions};
use crate::PathAssembleOptions;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Skin {
    pub assemble: Option<PathAssembleOptions>,
    pub render: Option<PartialRenderOptions>,
}


impl Skin {
    pub fn options(self) -> (PathAssembleOptions, RenderOptions) {
        (self.assemble.unwrap_or_default(), self.render.map_or_else(RenderOptions::default, RenderOptions::from))
    }

    #[cfg(feature = "json5")]
    #[inline]
    pub fn from_json5(s: &str) -> Result<Self, json5::Error> {
        json5::from_str(s)
    }

    #[cfg(feature = "serde_json")]
    #[inline]
    pub fn from_json(s: &str) -> Result<Self, serde_json::error::Error> {
        serde_json::from_str(s)
    }
}