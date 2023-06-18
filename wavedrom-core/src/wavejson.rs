//! The definitions for the WaveJson format.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{
    CycleEnumerationMarker, CycleOffset, CycleState, Figure, FigureSection,
    FigureSectionGroup, Signal,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveJson {
    pub signal: Vec<SignalItem>,
    pub head: Option<Head>,
    pub foot: Option<Foot>,
    pub config: Option<Config>,
    pub edge: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalItem {
    Group(Vec<SignalGroupItem>),
    Item(SignalObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalGroupItem {
    String(String),
    Item(SignalItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalObject {
    pub name: Option<String>,
    pub wave: Option<String>,
    pub data: Option<SignalData>,
    pub node: Option<String>,
    pub period: Option<f32>,
    pub phase: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalData {
    One(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Head {
    pub text: Option<String>,
    pub tick: Option<u32>,
    pub every: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foot {
    pub text: Option<String>,
    pub tock: Option<u32>,
    pub every: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hscale: Option<u16>,
    pub skin: Option<String>,
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
        let (title, top_cycle_marker) = if let Some(head) = value.head {
            let title = head.text;
            let top_cycle_marker = match (head.tick, head.every) {
                (Some(start), Some(every)) => Some(CycleEnumerationMarker::new(start, every)),
                (Some(start), None) => Some(CycleEnumerationMarker::new(start, 1)),
                (None, _) => None,
            };

            (title, top_cycle_marker)
        } else {
            (None, None)
        };
        let (footer, bottom_cycle_marker) = if let Some(foot) = value.foot {
            let footer = foot.text;
            let bottom_cycle_marker = match (foot.tock, foot.every) {
                (Some(start), Some(every)) => Some(CycleEnumerationMarker::new(start, every)),
                (Some(start), None) => Some(CycleEnumerationMarker::new(start, 1)),
                (None, _) => None,
            };

            (footer, bottom_cycle_marker)
        } else {
            (None, None)
        };

        let hscale = value.config.and_then(|config| config.hscale).unwrap_or(1);

        let sections = value
            .signal
            .into_iter()
            .map(FigureSection::from)
            .collect::<Vec<FigureSection>>();

        let mut edges = Vec::new();

        if let Some(edge) = value.edge {
            for e in edge {
                if let Ok(def) = e.parse() {
                    edges.push(def);
                }
            }
        }

        Figure::with(
            title,
            footer,
            top_cycle_marker,
            bottom_cycle_marker,
            hscale,
            sections,
            edges,
        )
    }
}

impl From<SignalItem> for FigureSection {
    fn from(signal: SignalItem) -> Self {
        match signal {
            SignalItem::Group(items) => {
                let mut label = None;

                let items = items
                    .into_iter()
                    .filter_map(|item| match item {
                        SignalGroupItem::String(s) => {
                            if label.is_none() {
                                label = Some(s);
                            }

                            None
                        }
                        SignalGroupItem::Item(line) => Some(FigureSection::from(line)),
                    })
                    .collect::<Vec<FigureSection>>();

                FigureSection::Group(FigureSectionGroup(label, items))
            }
            SignalItem::Item(item) => FigureSection::Signal(Signal::from(item)),
        }
    }
}

impl From<SignalObject> for Signal {
    fn from(item: SignalObject) -> Self {
        let name = item.name.unwrap_or_default();
        let cycles = item
            .wave
            .unwrap_or_default()
            .chars()
            .map(|c| CycleState::from(c))
            .collect();
        let data = item
            .data
            .map_or_else(Vec::new, |signal_data| match signal_data {
                SignalData::One(data) => data
                    .split(char::is_whitespace)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
                SignalData::Multiple(data) => data,
            });
        let node = item.node.unwrap_or_default();
        let period = item.period.map_or(0, |f| f.ceil() as u16);
        let phase = item.phase.map_or_else(CycleOffset::default, |f| {
            CycleOffset::try_from(f).unwrap_or_default()
        });

        Signal::with(name, cycles, data, node, period, phase)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde_json")]
    fn groups() {
        use super::*;

        let data = r#"
        {
            "signal": [
                [
                    "xyz",
                    { "name": "=", "wave": "=", "node": "a....." },
                    { "name": "0", "wave": "0" },
                    { "name": "1", "wave": "1" },
                    { "name": "2", "wave": "2" },
                    { "name": "z", "wave": "z", "node": ".....b" },
                    { "name": "x", "wave": "x" },
                    { "name": "p", "wave": "p" },
                    { "name": "P", "wave": "P" },
                    { "name": "n", "wave": "n" },
                    { "name": "N", "wave": "N" },
                    { "name": ".", "wave": "." },
                    { "name": "d", "wave": "d" },
                    { "name": "u", "wave": "u" }
                ],
                [
                    { "name": "0.|", "wave": "0.|" },
                    { "name": "1.|", "wave": "1.|" },
                    { "name": "2.|", "wave": "2.|" },
                    { "name": "z.|", "wave": "z.|" },
                    { "name": "x.|", "wave": "x.|" },
                    { "name": "p.|", "wave": "p.|" },
                    { "name": "P.|", "wave": "P.|" },
                    { "name": "n.|", "wave": "n.|" },
                    { "name": "N.|", "wave": "N.|" },
                    { "name": "..|", "wave": "..|" },
                    { "name": "d.|", "wave": "d.|" },
                    { "name": "u.|", "wave": "u.|" }
                ],
                [
                    { "name": "0001020z0x0p0P0n0N0.0", "wave": "0001020z0x0p0P0n0N0.0d0u0" },
                    { "name": "1011121z1x1p1P1n1N1.1", "wave": "1011121z1x1p1P1n1N1.1d1u1" },
                    { "name": "2021222z2x2p2P2n2N2.2", "wave": "2021222z2x2p2P2n2N2.2d2u2" },
                    { "name": "z0z1z2zzzxzpzPznzNz.z", "wave": "z0z1z2zzzxzpzPznzNz.zdzuz" },
                    { "name": "x0x1x2xzxxxpxPxnxNx.x", "wave": "x0x1x2xzxxxpxPxnxNx.xdxux" },
                    { "name": "p0p1p2pzpxpppPpnpNp.p", "wave": "p0p1p2pzpxpppPpnpNp.pdpup" },
                    { "name": "P0P1P2PzPxPpPPPnPNP.P", "wave": "P0P1P2PzPxPpPPPnPNP.PdPuP" },
                    { "name": "n0n1n2nznxnpnPnnnNn.n", "wave": "n0n1n2nznxnpnPnnnNn.ndnun" },
                    { "name": "N0N1N2NzNxNpNPNnNNN.N", "wave": "N0N1N2NzNxNpNPNnNNN.NdNuN" },
                    { "name": "u0u1u2uzuxupuPunuNu.u", "wave": "u0u1u2uzuxupuPunuNu.uduuu" },
                    { "name": "d0d1d2dzdxdpdPdndNd.d", "wave": "d0d1d2dzdxdpdPdndNd.dddud" }
                ],
                [
                    { "name": "0123456789=zx", "wave": "0123456789=zx" },
                    {
                        "name": "02....3...0",
                        "wave": "02....3...0",
                        "data": [
                            "0xDEAD",
                            "0xBEEF"
                        ]
                    },
                    {
                        "name": "2.2.2.2.",
                        "wave": "2.2.2.2.",
                        "data": [
                            "A",
                            "B",
                            "C",
                            "D"
                        ]
                    }
                ],
                [
                    { "name": "period 2, phase 0.5", "wave": "p", "period": 2, "phase": 0.5 }
                ]
            ],
            "head": {
                "text": "Interaction Test Figure",
                "tick": 3
            },
            "foot": {
                "text": "Some Footer Text",
                "tock": 42,
                "every": 3
            },
            "config": {
                "hscale": 1
            },
            "edge": ["a<->b xyz"]
        }
        "#;

        let wavejson: WaveJson = serde_json::from_str(data).unwrap();
        let figure: Figure = wavejson.try_into().unwrap();

        let rendered = figure.assemble();

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("test.svg")
            .unwrap();

        rendered.write_svg(&mut file).unwrap();
    }
}
