use serde::{Deserialize, Serialize};

use crate::{Figure, Wave};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveJson {
    pub signal: Vec<Signal>,
}

impl Into<Figure> for WaveJson {
    fn into(self) -> Figure {
        Figure(
            self.signal
                .into_iter()
                .map(|signal| match signal {
                    Signal::Group(_, _) => unimplemented!(),
                    Signal::Item(item) => item.into(),
                })
                .collect(),
        )
    }
}

impl Into<Wave> for SignalItem {
    fn into(self) -> Wave {
        Wave {
            name: self.name.unwrap_or_default(),
            cycles: self.wave.unwrap_or_default().parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Signal {
    Group(Option<String>, Vec<Signal>),
    Item(SignalItem),
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
    fn load_basic() {
        let data = r#"
        {
            "signal": [
                { "name": "abc", "wave": "123" }
            ]
        }
        "#;

        let wavejson: WaveJson = serde_json::from_str(data).unwrap();

        dbg!(wavejson);
        assert!(false);
    }

    #[test]
    fn to_svg() {
        use crate::svg::ToSvg;

        let data = r#"
        {
            "signal": [
                { "name": "abc", "wave": "120..." }
            ]
        }
        "#;

        let wavejson: WaveJson = serde_json::from_str(data).unwrap();
        let figure: Figure = wavejson.into();

        let rendered = figure.render().unwrap();

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("test.svg")
            .unwrap();

        rendered.write_svg(&mut file).unwrap();

        assert!(false);
    }
}