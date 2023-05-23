#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub u32, pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub location: Point,
    pub dimension: Point,
}

impl Point {
    pub fn x(self) -> u32 {
        self.0
    }
    pub fn y(self) -> u32 {
        self.1
    }
}

impl Rect {
    pub fn x(&self) -> u32 {
        self.location.0
    }
    pub fn y(&self) -> u32 {
        self.location.1
    }
    pub fn w(&self) -> u32 {
        self.dimension.0
    }
    pub fn h(&self) -> u32 {
        self.dimension.1
    }
}
