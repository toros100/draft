// yes i had to, no i could not just have used an existing crate

use std::ops::{Add, Mul, Sub};

pub fn point2(x: f64, y: f64) -> Point2 {
    Point2 { x, y }
}

pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

pub fn polar(dist: f64, angle: f64) -> Polar {
    Polar { dist, angle }
}

pub struct CubicBezier {
    p_0: Point2,
    p_1: Point2,
    p_2: Point2,
    p_3: Point2,
}

// TODO: use adaptive step count
const BEZIER_STEPS: usize = 1000;

impl CubicBezier {
    fn approx_length(&self) -> f64 {
        let mut acc = 0f64;
        let mut first = self.at(0.);
        for i in 1..=BEZIER_STEPS {
            let t = (i as f64) / (BEZIER_STEPS as f64);
            let next = self.at(t);
            acc += first.dist(next);
            first = next;
        }
        acc
    }

    fn at(&self, t: f64) -> Point2 {
        debug_assert!(0. <= t);
        debug_assert!(t <= 1.);

        let p_0: Vec2 = self.p_0.as_vec();
        let p_1: Vec2 = self.p_1.as_vec();
        let p_2: Vec2 = self.p_2.as_vec();
        let p_3: Vec2 = self.p_3.as_vec();

        ((1. - t).powi(3) * p_0
            + 3. * (1. - t).powi(2) * t * p_1
            + 3. * (1. - t) * t.powi(2) * p_2
            + t.powi(3) * p_3)
            .as_point()
    }

    fn point_on(&self, dist_from_start: f64) -> Point2 {
        // a lot of room to improve lol
        let mut acc = 0f64;
        let mut p = self.at(0.);
        for i in 1..=BEZIER_STEPS {
            if acc >= dist_from_start {
                return p;
            }
            let t = (i as f64) / (BEZIER_STEPS as f64);
            let next = self.at(t);
            acc += p.dist(next);
            p = next;
        }
        p
    }
}

pub fn cubic_bezier_length(p_0: Point2, p_1: Point2, p_2: Point2, p_3: Point2) -> f64 {
    let c = CubicBezier { p_0, p_1, p_2, p_3 };
    c.approx_length()
}

pub fn point_on_cubic_bezier(
    p_0: Point2,
    p_1: Point2,
    p_2: Point2,
    p_3: Point2,
    dist: f64,
) -> Point2 {
    let c = CubicBezier { p_0, p_1, p_2, p_3 };
    c.point_on(dist)
}

#[derive(Clone, Copy, Debug)]
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
    fn as_vec(self) -> Vec2 {
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
    pub fn new(dist: f64, angle: f64) -> Self {
        Self { dist, angle }
    }
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
    fn as_point(self) -> Point2 {
        Point2 {
            x: self.x,
            y: self.y,
        }
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
