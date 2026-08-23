use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, approx_derive::RelativeEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}


impl Vec2 {
    pub fn into_point(self) -> Point2 {
        Point2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn try_normalize(self) -> Option<Self> {
        let n = 1. / self.norm();
        if n < EPS { None } else { Some(self.scale(n)) }
    }

    pub fn scale(mut self, f: f64) -> Self {
        self.x *= f;
        self.y *= f;
        self
    }

    pub fn into_polar(self) -> Polar {
        Polar {
            dist: self.norm(),
            angle: f64::atan2(self.y, self.x),
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, approx_derive::RelativeEq)]
pub struct Polar {
    pub dist: f64,
    pub angle: f64,
}

impl Polar {
    pub fn new(dist: f64, angle: f64) -> Self {
        Self { dist, angle }
    }
    pub fn into_vec(self) -> Vec2 {
        let x = self.dist * self.angle.cos();
        let y = self.dist * self.angle.sin();
        Vec2 { x, y }
    }
}

impl From<Vec2> for Polar {
    fn from(value: Vec2) -> Self {
        value.into_polar()
    }
}

impl From<Polar> for Vec2 {
    fn from(value: Polar) -> Self {
        value.into_vec()
    }
}


impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl Mul<Vec2> for f64 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs.scale(self)
    }
}

