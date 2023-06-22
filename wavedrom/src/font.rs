use std::marker::PhantomData;

#[cfg(feature = "embed_font")]
static EMBEDDED_HELVETICA: std::sync::OnceLock<ttf_parser::Face<'static>> = std::sync::OnceLock::new(); 

/// The font that is used by the svg assembler to calculate text widths.
///
/// The `embed_font` feature (that is enabled by default)` includes a [Helvetica][helvetica] [TTF
/// file][ttf] into the binary, which enables better width calculations even when using more exotic
/// characters. When this feature is disabled, only the width of [ASCII][ascii] characters is
/// property calculated. Other characters widths are overestimated.
///
/// [helvetica]: https://en.wikipedia.org/wiki/Helvetica
/// [ttf]: https://en.wikipedia.org/wiki/TrueType
/// [ascii]: https://en.wikipedia.org/wiki/ASCII
#[derive(Debug, Default, Clone, Copy)]
pub struct Font {
    _marker: PhantomData<()>,
}

impl Font {
    /// Get an upperbound on the text width for a given string `s` and `font_size`.
    ///
    /// Given a `font_size` of *x* units the width of the string `s` without letter spacing is
    /// `get_text_width(s, font_size)` units.
    pub fn get_text_width(&self, s: &str, font_size: u32) -> u32 {
        let width = f64::from(self.get_pts_text_width(s));

        let pts_per_em = f64::from(font_size) / f64::from(self.units_per_em());
        let width = width * pts_per_em;

        width.ceil() as u32
    }
}

#[cfg(not(feature = "embed_font"))]
impl Font {
    #[inline]
    fn units_per_em(&self) -> u16 {
        2048
    }

    fn get_pts_text_width(&self, s: &'_ str) -> u32 {
        static ADVANCE_LUT: [u16; 128] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 569, 569, 0, 0, 569, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 569, 569, 727, 1139, 1139, 1821, 1366, 391, 682, 682, 797, 1196, 569,
            682, 569, 569, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 569, 569,
            1196, 1196, 1196, 1139, 2079, 1366, 1366, 1479, 1479, 1366, 1251, 1593, 1479, 569,
            1024, 1366, 1139, 1706, 1479, 1593, 1366, 1593, 1479, 1366, 1251, 1479, 1366, 1933,
            1366, 1366, 1251, 569, 569, 569, 961, 1139, 682, 1139, 1139, 1024, 1139, 1139, 569,
            1139, 1139, 455, 455, 1024, 455, 1706, 1139, 1139, 1139, 1139, 682, 1024, 569, 1139,
            1024, 1479, 1024, 1024, 1024, 684, 532, 684, 1196, 0,
        ];

        s.chars()
            .map(|c| {
                if c.is_ascii() {
                    u32::from(ADVANCE_LUT[c as usize])
                } else {
                    2052
                }
            })
            .sum()
    }

    /// Get a string representing the Font Family of the font.
    #[inline]
    pub fn get_font_family_name(&self) -> Option<String> {
        Some("Helvetica".to_string())
    }
}

#[cfg(feature = "embed_font")]
impl Font {
    fn get_face(&self) -> &ttf_parser::Face<'static> {
        EMBEDDED_HELVETICA.get_or_init(|| {
            ttf_parser::Face::parse(include_bytes!("../helvetica.ttf"), 0).unwrap()
        })
    }

    #[inline]
    fn units_per_em(&self) -> u16 {
        self.get_face().units_per_em()
    }

    fn get_pts_text_width(&self, s: &'_ str) -> u32 {
        let face = self.get_face();

        s.chars()
            .map(|c| {
                face
                    .glyph_index(c)
                    .and_then(|g| face.glyph_hor_advance(g))
                    .map(u32::from)
                    .unwrap_or(face.global_bounding_box().width() as u32)
            })
            .sum()
    }

    /// Get a string representing the Font Family of the font.
    pub fn get_font_family_name(&self) -> Option<String> {
        self.get_face()
            .names()
            .into_iter()
            .find(|item| item.name_id == 1)
            .and_then(|name| {
                if !name.is_unicode() {
                    return None;
                }

                // Invalid UTF16 check
                if name.name.len() % 2 != 0 {
                    return None;
                }

                let utf16_bytes = name
                    .name
                    .chunks_exact(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .collect::<Vec<u16>>();

                String::from_utf16(&utf16_bytes).ok()
            })
    }
}

#[ignore]
#[test]
#[cfg(all(feature = "gen_lut", feature = "embed_font"))]
fn generate_lookup_table() {
    let face = ttf_parser::Face::parse(include_bytes!("../helvetica.ttf"), 0).unwrap();

    println!("[");
    for c in 0..128u8 {
        let c = c as char;
        println!(
            "\t{},",
            face.glyph_index(c)
                .and_then(|g| face.glyph_hor_advance(g))
                .map(u32::from)
                .unwrap_or(0)
        );
    }
    println!("]");
    println!("units_per_em: {:?}", face.units_per_em());
    println!("Rest Advance: {:?}", face.global_bounding_box().width());

    assert!(false);
}