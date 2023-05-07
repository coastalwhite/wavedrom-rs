use ttf_parser::{Face, OutlineBuilder, Rect};

pub struct TextPath {
    data: String,
    bounding_box: Rect,
}

impl TextPath {
    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn bounding_box(&self) -> Rect {
        self.bounding_box
    }
}

pub(crate) struct GlyphPathBuilder<'a> {
    x: f32,
    y: f32,
    data: &'a mut String,
    units_per_em: f32,
    font_size: f32,
}

pub struct TextPathSettings<'a> {
    pub face: &'a Face<'a>,
    pub font_size: i16,
    pub letter_spacing: i16,
}

impl TextPath {
    pub fn build(content: &str, settings: &TextPathSettings, x: i16, y: i16) -> Self {
        let mut data = String::new();
        let x_min = x;
        let mut x = x;

        for c in content.chars() {
            let glyph_id = settings.face.glyph_index(c).unwrap();
            let mut outliner = GlyphPathBuilder::new(
                &mut data,
                x.into(),
                y.into(),
                settings.face.units_per_em().into(),
                settings.font_size.into(),
            );
            let Some(bounding_box) = settings.face.outline_glyph(glyph_id, &mut outliner) else {
                panic!("Failed to generate outline for glyph '{c}'");
            };
            x += ((bounding_box.width() * settings.font_size) as u16 / settings.face.units_per_em())
                as i16
                + settings.letter_spacing;
        }

        let bounding_box = Rect {
            x_min,
            y_min: y,
            x_max: x,
            y_max: y + settings.font_size,
        };

        TextPath { data, bounding_box }
    }
}

impl<'a> GlyphPathBuilder<'a> {
    pub(crate) fn new(
        data: &'a mut String,
        x: f32,
        y: f32,
        units_per_em: f32,
        font_size: f32,
    ) -> Self {
        Self {
            x,
            y,
            data,
            units_per_em,
            font_size,
        }
    }

    #[inline]
    fn correct_x(&self, x: f32) -> f32 {
        (x * self.font_size) / self.units_per_em
    }

    #[inline]
    fn correct_y(&self, y: f32) -> f32 {
        self.font_size - ((y * self.font_size) / self.units_per_em)
    }
}

impl<'a> OutlineBuilder for GlyphPathBuilder<'a> {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = self.correct_x(x) + self.x;
        let y = self.correct_y(y) + self.y;

        self.data.push_str(&format!("M{x} {y}"));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = self.correct_x(x) + self.x;
        let y = self.correct_y(y) + self.y;

        self.data.push_str(&format!("L{x} {y}"));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let x1 = self.correct_x(x1) + self.x;
        let y1 = self.correct_y(y1) + self.y;

        let x = self.correct_x(x) + self.x;
        let y = self.correct_y(y) + self.y;

        self.data.push_str(&format!("Q{x1} {y1},{x} {y}"));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let x1 = self.correct_x(x1) + self.x;
        let y1 = self.correct_y(y1) + self.y;

        let x2 = self.correct_x(x2) + self.x;
        let y2 = self.correct_y(y2) + self.y;

        let x = self.correct_x(x) + self.x;
        let y = self.correct_y(y) + self.y;

        self.data.push_str(&format!("C{x1} {y1},{x2} {y2},{x} {y}"));
    }

    fn close(&mut self) {
        self.data.push('Z');
    }
}
