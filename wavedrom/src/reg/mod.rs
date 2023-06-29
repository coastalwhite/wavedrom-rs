pub mod render;

pub struct RegisterFigure {
    lanes: Vec<Lane>,
}

pub enum FieldString {
    Text(String),
    Binary(u64),
}

pub struct Lane {
    bit_ranges: Vec<LaneBitRange>,
    start_bit: u32,
    width: u32,
}

pub struct LaneBitRange {
    name: Option<FieldString>,
    attributes: Vec<FieldString>,
    length: u32,
    variant: u32,
}

impl FromIterator<Lane> for RegisterFigure {
    fn from_iter<T: IntoIterator<Item = Lane>>(iter: T) -> Self {
        Self {
            lanes: <Vec<Lane>>::from_iter(iter),
        }
    }
}

impl RegisterFigure {
    pub fn with(lanes: Vec<Lane>) -> Self {
        Self { lanes }
    }
}

impl LaneBitRange {
    pub fn with(name: Option<FieldString>, attributes: Vec<FieldString>, length: u32, variant: u32) -> Self {
        Self {
            name,
            attributes,
            length,
            variant,
        }
    }

    pub fn new(length: u32) -> Self {
        Self {
            name: None,
            attributes: Vec::new(),
            length,
            variant: 0,
        }
    }

    pub fn new_padding(length: u32) -> Self {
        Self {
            name: None,
            attributes: Vec::new(),
            length,
            variant: 0,
        }
    }

    // pub fn name(mut self, name: impl Into<String>) -> Self {
    //     self.name = Some(name.into());
    //     self
    // }
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            bit_ranges: Vec::new(),
            width: 0,
            start_bit: 0,
        }
    }
}

impl Lane {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn padded(width: u32) -> Self {
        Self {
            bit_ranges: Vec::new(),
            start_bit: 0,
            width,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0
    }

    pub fn pad(mut self, padding: u32) -> Self {
        self.width += padding;
        self
    }

    pub fn start_bit(mut self, start_bit: u32) -> Self {
        self.start_bit = start_bit;
        self
    }

    pub fn add(mut self, bit_range: LaneBitRange) -> Self {
        self.width += bit_range.length;
        self.bit_ranges.push(bit_range);
        self
    }
}
