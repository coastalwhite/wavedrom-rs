use serde::{Deserialize, Serialize};

use crate::signal::markers::CycleEnumerationMarker;
use crate::signal::{CycleOffset, CycleState};
use crate::signal::{Signal, SignalFigure, SignalFigureSection, SignalFigureSectionGroup};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalJson {
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

impl From<SignalJson> for SignalFigure {
    fn from(value: SignalJson) -> Self {
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
            .map(SignalFigureSection::from)
            .collect::<Vec<SignalFigureSection>>();

        let mut edges = Vec::new();

        if let Some(edge) = value.edge {
            for e in edge {
                if let Ok(def) = e.parse() {
                    edges.push(def);
                }
            }
        }

        SignalFigure::with(
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

impl From<SignalItem> for SignalFigureSection {
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
                        SignalGroupItem::Item(line) => Some(SignalFigureSection::from(line)),
                    })
                    .collect::<Vec<SignalFigureSection>>();

                SignalFigureSection::Group(SignalFigureSectionGroup::new(label, items))
            }
            SignalItem::Item(item) => SignalFigureSection::Signal(Signal::from(item)),
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
            .map(CycleState::from)
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
