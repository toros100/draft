use std::ops::{Add, Mul, Sub};

mod kurbo_conv;

mod rect;
pub use rect::*;
mod curve;
pub use curve::*;
mod point;
pub use point::*;
mod vec;
pub use vec::*;

// reasonable?
pub const EPS: f64 = 1e-10;

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
