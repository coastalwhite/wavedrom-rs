use serde::{Deserialize, Serialize};

use crate::{Figure, Wave, WaveLine, WaveLineGroup};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveJson {
    pub signal: Vec<Signal>,
}

impl WaveJson {
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl TryFrom<WaveJson> for Figure {
    type Error = ();
    fn try_from(value: WaveJson) -> Result<Self, Self::Error> {
        Ok(Figure::from_lines(
            value.signal
                .into_iter()
                .map(WaveLine::try_from)
                .collect::<Result<Vec<WaveLine>, ()>>()?,
        ))
    }
}

impl TryFrom<Signal> for WaveLine {
    type Error = ();
    fn try_from(signal: Signal) -> Result<Self, Self::Error> {
        match signal {
            Signal::Group(items) => {
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
                        SignalGroupItem::Item(line) => Some(WaveLine::try_from(line)),
                    })
                    .collect::<Result<Vec<WaveLine>, ()>>()?;

                Ok(WaveLine::Group(WaveLineGroup(label, items)))
            }
            Signal::Item(item) => Ok(WaveLine::Wave(Wave::try_from(item)?)),
        }
    }
}

impl TryFrom<SignalItem> for Wave {
    type Error = ();

    fn try_from(item: SignalItem) -> Result<Self, Self::Error> {
        Ok(Wave {
            name: item.name.unwrap_or_default(),
            cycles: item.wave.unwrap_or_default().parse().map_err(|_| ())?,
            data: item
                .data
                .map_or_else(Vec::new, |signal_data| match signal_data {
                    SignalData::One(data) => vec![data],
                    SignalData::Multiple(data) => data,
                }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Signal {
    Group(Vec<SignalGroupItem>),
    Item(SignalItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalGroupItem {
    String(String),
    Item(Signal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalItem {
    pub name: Option<String>,
    pub wave: Option<String>,
    pub data: Option<SignalData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalData {
    One(String),
    Multiple(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups() {
        use crate::svg::ToSvg;

        let data = r#"
        {
            "signal": [
                [
                    "xyz",
                    { "name": "0", "wave": "0" },
                    { "name": "1", "wave": "1" },
                    { "name": "2", "wave": "2" },
                    { "name": "z", "wave": "z" },
                    { "name": "x", "wave": "x" },
                    { "name": "p", "wave": "p" },
                    { "name": "P", "wave": "P" },
                    { "name": "n", "wave": "n" },
                    { "name": "N", "wave": "N" }
                ],
                [
                    { "name": "0001020z0x0p0P0n0N0", "wave": "0001020z0x0p0P0n0N0" },
                    { "name": "1011121z1x1p0P0n0N0", "wave": "1011121z1x1p1P1n1N1" },
                    { "name": "2021222z2x2p2P2n2N2", "wave": "2021222z2x2p2P2n2N2" },
                    { "name": "z0z1z2zzzxzpzPznzNz", "wave": "z0z1z2zzzxzpzPznzNz" },
                    { "name": "x0x1x2xzxxxpxPxnxNx", "wave": "x0x1x2xzxxxpxPxnxNx" },
                    { "name": "p0p1p2pzpxpppPpnpNp", "wave": "p0p1p2pzpxpppPpnpNp" },
                    { "name": "P0P1P2PzPxPpPPPnPNP", "wave": "P0P1P2PzPxPpPPPnPNP" },
                    { "name": "n0n1n2nznxnpnPnnnNn", "wave": "n0n1n2nznxnpnPnnnNn" },
                    { "name": "N0N1N2NzNxNpNPNnNNN", "wave": "N0N1N2NzNxNpNPNnNNN" }
                ],
                [
                    { "name": "012345zx", "wave": "012345zx" },
                    {
                        "name": "02....3...0",
                        "wave": "02....3...0",
                        "data": [
                            "0xDEAD",
                            "0xBEEF"
                        ]
                    }
                ]
            ]
        }
        "#;


        let wavejson: WaveJson = serde_json::from_str(data).unwrap();
        let figure: Figure = wavejson.try_into().unwrap();

        let rendered = figure.assemble().unwrap();

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("test.svg")
            .unwrap();

        rendered.write_svg(&mut file).unwrap();
    }

    // #[test]
    // fn to_svg() {
    //     use crate::svg::ToSvg;
    //
    //     let data = r#"
    //     {
    //         "signal": [
    //             { "name": "0", "wave": "0" },
    //             { "name": "1", "wave": "1" },
    //             { "name": "2", "wave": "2" },
    //             { "name": "z", "wave": "z" },
    //             { "name": "x", "wave": "x" },
    //             {},
    //             { "name": "0001020z0x0", "wave": "0001020z0x0" },
    //             { "name": "1011121z1x1", "wave": "1011121z1x1" },
    //             { "name": "2021222z2x2", "wave": "2021222z2x2" },
    //             { "name": "z0z1z2zzzxz", "wave": "z0z1z2zzzxz" },
    //             { "name": "x0x1x2xzxxx", "wave": "x0x1x2xzxxx" },
    //             {},
    //             { "name": "012345zx", "wave": "012345zx" },
    //             {
    //                 "name": "02....3...0",
    //                 "wave": "02....3...0",
    //                 "data": [
    //                     "0xDEAD",
    //                     "0xBEEF"
    //                 ]
    //             }
    //         ]
    //     }
    //     "#;
    //
    //     let wavejson: WaveJson = serde_json::from_str(data).unwrap();
    //     let figure: Figure = wavejson.into();
    //
    //     let rendered = figure.assemble().unwrap();
    //
    //     let mut file = std::fs::OpenOptions::new()
    //         .read(true)
    //         .write(true)
    //         .truncate(true)
    //         .create(true)
    //         .open("test.svg")
    //         .unwrap();
    //
    //     rendered.write_svg(&mut file).unwrap();
    // }
}
