// yes i had to, no i could not just have used an existing crate

use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl From<crate::slint_generatedMainWindow::WorldPos> for Point2 {
    fn from(value: crate::slint_generatedMainWindow::WorldPos) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl From<Point2> for crate::slint_generatedMainWindow::WorldPos {
    fn from(value: Point2) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
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
    pub fn new(x: f64, y: f64) -> Self {
        Point2 { x, y }
    }

    pub fn dist(self, other: Point2) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
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

#[derive(Clone, Copy, Default, Debug)]
pub struct Polar {
    pub dist: f64,
    pub angle: f64,
}

impl Polar {
    pub fn to_vec(self) -> Vec2 {
        let x = self.dist * self.angle.cos();
        let y = self.dist * self.angle.sin();
        Vec2 { x, y }
    }
}

impl From<Polar> for Vec2 {
    fn from(value: Polar) -> Self {
        value.to_vec()
    }
}

impl Add<Polar> for Point2 {
    type Output = Point2;
    fn add(self, rhs: Polar) -> Self::Output {
        self + rhs.to_vec()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
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

// reasonable?
pub const EPS: f64 = 1e-10;

impl Vec2 {
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

    pub fn to_polar(self) -> Polar {
        Polar {
            dist: self.norm(),
            angle: f64::atan2(self.y, self.x),
        }
    }
}

impl From<Vec2> for Polar {
    fn from(value: Vec2) -> Self {
        value.to_polar()
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

impl Sub for Point2 {
    type Output = Vec2;
    fn sub(self, rhs: Point2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
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
