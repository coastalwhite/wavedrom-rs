use serde::{Deserialize, Serialize};

use crate::reg::{Register, RegisterBitRange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegJson {
    pub reg: Vec<RegItem>,
    pub config: Option<RegJsonConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegItem {
    bits: u32,
    name: Option<String>,
    attr: Option<RegItemAttribute>,
    #[serde(rename = "type")]
    variant: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegJsonConfig {
    vspace: Option<u32>,
    hspace: Option<u32>,
    lanes: Option<u32>,
    bits: Option<u32>,
    fontsize: Option<u32>,
    fontweight: Option<u32>,
    fontfamily: Option<u32>,
    compact: Option<u32>,
    hflip: Option<u32>,
    vflip: Option<u32>,
    uneven: Option<bool>,
    offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegItemAttribute {
    One(String),
    Multiple(Vec<String>),
}

impl From<RegJson> for Register {
    fn from(value: RegJson) -> Self {
        let mut register = Register::new();

        for item in value.reg {
            let attributes = item.attr.map_or_else(Vec::default, |attr| match attr {
                RegItemAttribute::One(s) => vec![s],
                RegItemAttribute::Multiple(strs) => strs,
            });
            let bit_range =
                RegisterBitRange::with(item.name, attributes, item.bits, item.variant.unwrap_or(0));
            register = register.add(bit_range);
        }

        register
    }
}
