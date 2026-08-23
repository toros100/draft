use super::*;

#[derive(Default, Clone, Copy, Debug, PartialEq, approx_derive::RelativeEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Add for Point2 {
    type Output = Point2;
    fn add(self, rhs: Self) -> Self::Output {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Point2 {
    pub fn into_vec(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn new(x: f64, y: f64) -> Self {
        Point2 { x, y }
    }

    pub fn dist(self, other: Point2) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn dist_limit(self, other: Point2, limit: f64) -> Option<f64> {
        let d = self.dist(other);
        if d < limit { Some(d) } else { None }
    }

    pub fn angle(self, other: Point2) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        f64::atan2(dy, dx)
    }

    pub fn vec_to(self, other: Point2) -> Vec2 {
        other.sub(self)
    }
}

impl Sub for Point2 {
    type Output = Vec2;
    fn sub(self, rhs: Point2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<Polar> for Point2 {
    type Output = Point2;
    fn add(self, rhs: Polar) -> Self::Output {
        self + rhs.into_vec()
    }
}

impl Add<Vec2> for Point2 {
    type Output = Point2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Point2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Point2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Point2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

