//! The definitions for the WaveJson format.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use self::signal::SignalJson;

use crate::Figure;

pub mod signal;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WaveJson {
    Signal(SignalJson),
}

impl Figure {
    #[cfg(feature = "serde_json")]
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        WaveJson::from_json(s).map(Into::into)
    }

    #[cfg(feature = "json5")]
    pub fn from_json5(s: &str) -> Result<Self, json5::Error> {
        WaveJson::from_json5(s).map(Into::into)
    }
}

impl WaveJson {
    #[cfg(feature = "serde_json")]
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    #[cfg(feature = "json5")]
    pub fn from_json5(s: &str) -> Result<Self, json5::Error> {
        json5::from_str(s)
    }
}

impl From<WaveJson> for Figure {
    fn from(value: WaveJson) -> Self {
        match value {
            WaveJson::Signal(signal_json) => Figure::Signal(signal_json.into()),
        }
    }
}
