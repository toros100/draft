use super::*;

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    // top left corner
    pub fn pos(&self) -> Point2 {
        point2(self.x, self.y)
    }

    pub fn size(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    pub fn grow(self, f: f64) -> Self {
        Self {
            x: self.x - f,
            y: self.y + f,
            width: self.width + 2. * f,
            height: self.height + 2. * f,
        }
    }

    pub fn contains(&self, p: Point2) -> bool {
        self.x <= p.x && p.x <= self.x + self.width && self.y <= p.y && p.y <= self.y + self.height
    }
}
