pub mod render;

pub struct Register {
    bit_ranges: Vec<RegisterBitRange>,
    width: u32,
}

pub struct RegisterBitRange {
    name: Option<String>,
    attributes: Vec<String>,
    length: u32,
    variant: u32,
}

impl RegisterBitRange {
    pub fn with(name: Option<String>, attributes: Vec<String>, length: u32, variant: u32) -> Self {
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

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Register {
    pub fn new() -> Self {
        Self {
            bit_ranges: Vec::new(),
            width: 0,
        }
    }

    pub fn add(mut self, bit_range: RegisterBitRange) -> Self {
        self.width += bit_range.length;
        self.bit_ranges.push(bit_range);
        self
    }
}
