use super::*;

impl From<Point2> for vello::kurbo::Point {
    fn from(value: Point2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<vello::kurbo::Point> for Point2 {
    fn from(value: vello::kurbo::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Vec2> for vello::kurbo::Vec2 {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<vello::kurbo::Vec2> for Vec2 {
    fn from(value: vello::kurbo::Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
