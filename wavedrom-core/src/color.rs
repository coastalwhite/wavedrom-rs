use std::fmt::Display;
use std::str::FromStr;

/// An Red, Green, Blue color structure that can be parsed from and to a CSS compatible string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component of the [`Color`] RGB value
    pub red: u8,
    /// Green component of the [`Color`] RGB value
    pub green: u8,
    /// Blue component of the [`Color`] RGB value
    pub blue: u8,
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
enum SerdeColor {
    String(String),
    Array([u8; 3]),
    Rgb { red: u8, green: u8, blue: u8 },
}

#[cfg(feature = "serde")]
impl serde::Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerdeColor::Rgb {
            red: self.red,
            green: self.green,
            blue: self.blue,
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serde_color = SerdeColor::deserialize(deserializer)?;

        Ok(match serde_color {
            SerdeColor::String(s) => Color::from_str(&s).map_err(|at| {
                serde::de::Error::custom(format!("Failed to parse color invalid value at {at}"))
            })?,
            SerdeColor::Array([red, green, blue]) | SerdeColor::Rgb { red, green, blue } => {
                Self { red, green, blue }
            }
        })
    }
}

fn can_be_shortened(b: u8) -> bool {
    ((b & 0xF0) >> 4) == (b & 0xF)
}

impl FromStr for Color {
    type Err = usize;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") {
            return Err(0);
        }

        let s = &s[1..];

        if s.len() < 3 {
            return Err(s.len() + 1);
        }

        if s.len() == 3 {
            if s.chars().count() != 3 {
                return Err(1);
            };

            let red = &s[0..1];
            let green = &s[1..2];
            let blue = &s[2..3];

            let red = u8::from_str_radix(red, 16).map_err(|_| 1usize)?;
            let green = u8::from_str_radix(green, 16).map_err(|_| 2usize)?;
            let blue = u8::from_str_radix(blue, 16).map_err(|_| 3usize)?;

            let red = (red << 4) | red;
            let green = (green << 4) | green;
            let blue = (blue << 4) | blue;

            return Ok(Self { red, green, blue });
        }

        if s.len() < 6 {
            return Err(s.len() + 1);
        }

        if s.len() > 6 {
            return Err(7);
        }

        if s.chars().count() != 6 {
            return Err(1);
        };

        let red = &s[0..2];
        let green = &s[2..4];
        let blue = &s[4..6];

        let red = u8::from_str_radix(red, 16).map_err(|_| 1usize)?;
        let green = u8::from_str_radix(green, 16).map_err(|_| 3usize)?;
        let blue = u8::from_str_radix(blue, 16).map_err(|_| 5usize)?;

        Ok(Self { red, green, blue })
    }
}

impl Color {
    /// The color white. Namely rgb(255, 255, 255)
    pub const WHITE: Self = Color { red: 0xFF, green: 0xFF, blue: 0xFF };
    /// The color black. Namely rgb(0, 0, 0)
    pub const BLACK: Self = Color { red: 0x0, green: 0x0, blue: 0x0 };
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if can_be_shortened(self.red) && can_be_shortened(self.green) && can_be_shortened(self.blue)
        {
            write!(
                f,
                "#{:X}{:X}{:X}",
                self.red & 0xF,
                self.green & 0xF,
                self.blue & 0xF,
            )
        } else {
            write!(f, "#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
        }
    }
}
