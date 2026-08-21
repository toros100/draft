use std::ops::{Add, Mul, Sub};

mod curve;
pub use curve::*;

// TODO: refactor (Line type?)
pub fn closest_point_on_line_segment(a: Point2, b: Point2, q: Point2) -> (Point2, f64) {
    let ab = b - a;
    let aq = q - a;
    let t = (aq.dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    (a + t * ab, t)
}

pub fn closest_point_on_beam(a: Point2, b: Point2, q: Point2) -> (Point2, f64) {
    let ab = b - a;
    let aq = q - a;
    let t = aq.dot(ab) / ab.dot(ab);
    (a + t * ab, t)
}

pub fn closest_point_on_curve(c: CubicBezier, q: Point2) -> (Point2, f64) {
    let t = c.closest_to_at_approx(q);
    (c.at(t), t)
}

pub fn point2(x: f64, y: f64) -> Point2 {
    Point2 { x, y }
}

pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

pub fn polar(dist: f64, angle: f64) -> Polar {
    Polar { dist, angle }
}

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
    fn into_vec(self) -> Vec2 {
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

impl Add<Polar> for Point2 {
    type Output = Point2;
    fn add(self, rhs: Polar) -> Self::Output {
        self + rhs.into_vec()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, approx_derive::RelativeEq)]
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
    fn into_point(self) -> Point2 {
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
