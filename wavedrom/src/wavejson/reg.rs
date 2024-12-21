use serde::{Deserialize, Serialize};

use crate::reg::{Lane, LaneBitRange, RegisterFigure, FieldString};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RegJson {
    pub reg: Vec<RegItem>,
    pub config: Option<RegJsonConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RegItem {
    bits: u32,
    name: Option<RegFieldString>,
    attr: Option<RegItemAttribute>,
    #[serde(rename = "type")]
    variant: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RegFieldString {
    Text(String),
    Binary(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RegJsonConfig {
    vspace: Option<u32>,
    hspace: Option<u32>,
    lanes: Option<u32>,
    bits: Option<u32>,
    fontsize: Option<u32>,
    fontweight: Option<u32>,
    fontfamily: Option<u32>,
    compact: Option<u32>,
    hflip: Option<bool>,
    vflip: Option<bool>,
    uneven: Option<bool>,
    offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(untagged)]
pub enum RegItemAttribute {
    One(RegFieldString),
    Multiple(Vec<RegFieldString>),
}

fn create_lane_bitrange(num_bits: u32, item: &RegItem) -> LaneBitRange {
    let attributes = item
        .attr
        .as_ref()
        .map_or_else(Vec::default, |attr| match attr {
            RegItemAttribute::One(s) => vec![s.clone().into()],
            RegItemAttribute::Multiple(strs) => strs.iter().cloned().map(Into::into).collect(),
        });

    LaneBitRange::with(
        item.name.clone().map(Into::into),
        attributes,
        num_bits,
        item.variant.unwrap_or(0),
    )
}

impl From<RegFieldString> for FieldString {
    fn from(value: RegFieldString) -> Self {
        match value {
            RegFieldString::Text(s) => FieldString::Text(s),
            RegFieldString::Binary(b) => FieldString::Binary(b),
        }
    }
}

impl From<RegJson> for RegisterFigure {
    fn from(value: RegJson) -> Self {
        let num_lanes = value
            .config
            .as_ref()
            .and_then(|config| config.lanes)
            .unwrap_or(1);
        let num_lanes = u32::max(1, num_lanes);

        let num_bits = value
            .config
            .as_ref()
            .and_then(|config| config.bits)
            .unwrap_or_else(|| value.reg.iter().map(|range| range.bits).sum());

        let mut lanes = if num_lanes == 1 {
            let mut lane = Lane::new();

            let mut allocated_bits = 0;
            for item in value.reg {
                let bits = if allocated_bits + item.bits > num_bits {
                    num_bits - allocated_bits
                } else {
                    item.bits
                };

                allocated_bits += bits;

                let bit_range = create_lane_bitrange(bits, &item);

                lane = lane.add(bit_range);
            }

            if allocated_bits != num_bits {
                lane = lane.pad(num_bits - allocated_bits);
            }

            vec![lane]
        } else {
            let bits_per_lane = if num_lanes > num_bits {
                1
            } else if num_bits % num_lanes == 0 {
                num_bits / num_lanes
            } else {
                num_bits / num_lanes + 1
            };

            let mut lanes = Vec::with_capacity(num_lanes as usize);

            let mut lane = Lane::new();

            let mut figure_allocated_bits = 0;
            let mut lane_allocated_bits = 0;
            for item in value.reg {
                if figure_allocated_bits == num_bits {
                    break;
                }

                let mut unprocessed_bits = item.bits;
                while unprocessed_bits != 0 {
                    let bits_to_lane =
                        u32::min(unprocessed_bits, bits_per_lane - lane_allocated_bits);
                    let bits_to_lane = u32::min(bits_to_lane, num_bits - figure_allocated_bits);

                    let is_bit_range_split = (bits_to_lane != unprocessed_bits) && unprocessed_bits != 0;
                    let is_lane_full = lane_allocated_bits == bits_per_lane;

                    let bit_range = create_lane_bitrange(bits_to_lane, &item);

                    lane = lane.add(bit_range);
                    unprocessed_bits -= bits_to_lane;
                    lane_allocated_bits += bits_to_lane;
                    figure_allocated_bits += bits_to_lane;

                    let is_figure_full = figure_allocated_bits == num_bits;
                    if is_figure_full {
                        break;
                    }

                    if is_bit_range_split || is_lane_full {
                        lanes.push(std::mem::take(&mut lane));
                        lane = lane.start_bit((lanes.len() as u32) * bits_per_lane);
                        lane_allocated_bits = 0;
                    }
                }
            }

            if lanes.len() as u32 != num_lanes {
                if lane_allocated_bits != 0 {
                    lane = lane.pad(bits_per_lane - lane_allocated_bits);
                    lanes.push(lane);
                }

                let needed_lanes = num_lanes - (lanes.len() as u32);
                for _ in 0..needed_lanes {
                    lanes.push(
                        Lane::padded(bits_per_lane).start_bit((lanes.len() as u32) * bits_per_lane),
                    );
                }
            }

            lanes
        };

        let vflip = value
            .config
            .as_ref()
            .and_then(|config| config.vflip)
            .unwrap_or_default();
        if vflip {
            lanes.reverse();
        }

        let mut figure = RegisterFigure::with(lanes);
    
        if let Some(vspace) = value.config.as_ref().and_then(|c| c.vspace) {
            figure = figure.vspace(vspace);
        }
        if let Some(hspace) = value.config.as_ref().and_then(|c| c.hspace) {
            figure = figure.hspace(hspace);
        }

        figure
    }
}
