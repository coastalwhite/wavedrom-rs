#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Rect {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn new_with_vertices(x1: f64, y1: f64, x2: f64, y2: f64) -> Rect {
        Self {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }
}
