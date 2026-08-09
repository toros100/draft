use super::*;

#[derive(Default, Clone, Copy, Debug)]
pub struct CubicBezier {
    pub p_0: Point2,
    pub p_1: Point2,
    pub p_2: Point2,
    pub p_3: Point2,
}

// TODO: use adaptive step count (more likely a different algorithm)
const BEZIER_STEPS: usize = 1000;

pub fn curve(p_0: Point2, p_1: Point2, p_2: Point2, p_3: Point2) -> CubicBezier {
    CubicBezier { p_0, p_1, p_2, p_3 }
}

impl CubicBezier {
    pub fn new(p_0: Point2, p_1: Point2, p_2: Point2, p_3: Point2) -> CubicBezier {
        CubicBezier { p_0, p_1, p_2, p_3 }
    }

    pub fn approx_length(&self) -> f64 {
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

    pub fn approx_length_steps<const STEPS: usize>(&self) -> f64 {
        let mut acc = 0f64;
        let mut first = self.at(0.);
        for i in 1..=STEPS {
            let t = (i as f64) / (STEPS as f64);
            let next = self.at(t);
            acc += first.dist(next);
            first = next;
        }
        acc
    }

    pub fn at(&self, t: f64) -> Point2 {
        debug_assert!(0. <= t);
        debug_assert!(t <= 1.);
        let t = t.clamp(0., 1.);

        ((1. - t).powi(3) * self.p_0.as_vec()
            + 3. * (1. - t).powi(2) * t * self.p_1.as_vec()
            + 3. * (1. - t) * t.powi(2) * self.p_2.as_vec()
            + t.powi(3) * self.p_3.as_vec())
        .as_point()
    }

    pub fn point_on(&self, dist_from_start: f64) -> Point2 {
        // a lot of room to improve lol

        if dist_from_start <= 0. {
            return self.p_0;
        }

        let mut acc = 0f64;
        let mut p = self.at(0.);
        for i in 1..BEZIER_STEPS {
            let t = (i as f64) / ((BEZIER_STEPS - 1) as f64);
            let next = self.at(t);
            let d = p.dist(next);

            if acc + d >= dist_from_start {
                return p;
            }
            acc += d;
            p = next;
        }
        p
    }

    // cf. de casteljau
    pub fn split_at(&self, t: f64) -> (CubicBezier, CubicBezier) {
        debug_assert!(0. <= t);
        debug_assert!(t <= 1.);
        let t = t.clamp(0., 1.);

        let b_0_0 = self.p_0.as_vec();
        let b_1_0 = self.p_1.as_vec();
        let b_2_0 = self.p_2.as_vec();
        let b_3_0 = self.p_3.as_vec();

        let b_0_1 = b_0_0 * (1. - t) + b_1_0 * t;
        let b_1_1 = b_1_0 * (1. - t) + b_2_0 * t;
        let b_2_1 = b_2_0 * (1. - t) + b_3_0 * t;

        let b_0_2 = b_0_1 * (1. - t) + b_1_1 * t;
        let b_1_2 = b_1_1 * (1. - t) + b_2_1 * t;

        let b_0_3 = b_0_2 * (1. - t) + b_1_2 * t;

        let first = CubicBezier {
            p_0: b_0_0.as_point(),
            p_1: b_0_1.as_point(),
            p_2: b_0_2.as_point(),
            p_3: b_0_3.as_point(),
        };

        let second = CubicBezier {
            p_0: b_0_3.as_point(),
            p_1: b_1_2.as_point(),
            p_2: b_2_1.as_point(),
            p_3: b_3_0.as_point(),
        };

        (first, second)
    }

    fn bounding_rect(&self) -> Rect {
        let x_min = self.p_0.x.min(self.p_1.x).min(self.p_2.x).min(self.p_3.x);
        let x_max = self.p_0.x.max(self.p_1.x).max(self.p_2.x).max(self.p_3.x);
        let y_min = self.p_0.y.min(self.p_1.y).min(self.p_2.y).min(self.p_3.y);
        let y_max = self.p_0.y.max(self.p_1.y).max(self.p_2.y).max(self.p_3.y);

        Rect::new(x_min, y_max, x_max - x_min, y_max - y_min)
    }

    pub fn closest_to_at_approx(&self, target: Point2, limit: f64) -> Option<f64> {
        // TODO: not very nice

        if !self.bounding_rect().grow(limit).contains(target) {
            None
        } else {
            let mut t_min = None;
            let mut d_min = f64::INFINITY;
            for i in 0..BEZIER_STEPS {
                let t = (i as f64) / ((BEZIER_STEPS - 1) as f64);
                let d = self.at(t).dist(target);
                if d < d_min {
                    d_min = d;
                    t_min = Some(t)
                }
            }
            t_min
        }
    }

    pub fn dist(&self, target: Point2, limit: f64) -> Option<f64> {
        match self.closest_to_at_approx(target, limit) {
            Some(t) if self.at(t).dist(target) >= limit => None,
            o => o,
        }
    }
}

struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Rect {
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn grow(self, f: f64) -> Self {
        Self {
            x: self.x - f,
            y: self.y + f,
            width: self.width + 2. * f,
            height: self.height + 2. * f,
        }
    }

    fn contains(&self, p: Point2) -> bool {
        self.x <= p.x && p.x <= self.x + self.width && p.y <= self.y && self.y - self.height <= p.y
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    // right now i just want to test that the implementations are probably generally correct
    // seems less hacky than hand-picking epsilon per test lol
    const TEST_EPS: f64 = 1e-10;

    use super::*;

    #[test]
    fn length_straight() {
        let c = curve(
            point2(0., 0.),
            point2(0., 0.),
            point2(0., 100.),
            point2(0., 100.),
        );

        let c_len = c.approx_length();

        assert_relative_eq!(c_len, 100.);

        let d = curve(
            point2(0., 0.),
            point2(0., 0.),
            point2(100., 100.),
            point2(100., 100.),
        );

        let d_len = d.approx_length();

        let calculated_len = 20000f64.sqrt();

        assert_relative_eq!(d_len, calculated_len, epsilon = TEST_EPS);
    }

    #[test]
    fn length_zero() {
        let c = curve(
            point2(5., 5.),
            point2(5., 5.),
            point2(5., 5.),
            point2(5., 5.),
        );

        let c_len = c.approx_length();

        assert_relative_eq!(c_len, 0., epsilon = TEST_EPS);
    }

    #[test]
    fn split_preserves_length() {
        let c = curve(
            point2(0., 0.),
            point2(0., 50.),
            point2(100., 30.),
            point2(100., 0.),
        );

        let (a, b) = c.split_at(0.5);

        let a_len = a.approx_length_steps::<500>();
        let b_len = b.approx_length_steps::<500>();
        let c_len = c.approx_length_steps::<1000>();

        assert_relative_eq!(a_len + b_len, c_len, epsilon = TEST_EPS);
    }

    #[test]
    fn split_preserves_at() {
        const SAMPLES: usize = 1000;

        let c = curve(
            point2(0., 0.),
            point2(0., 50.),
            point2(100., 30.),
            point2(100., 0.),
        );

        let (a, b) = c.split_at(0.5);

        for i in 0..SAMPLES {
            let t = (i as f64) / ((SAMPLES - 1) as f64);
            let c_val = c.at(t);

            let comparison_val = if t <= 0.5 {
                a.at(2. * t)
            } else {
                b.at(2. * (t - 0.5))
            };

            assert_relative_eq!(c_val, comparison_val, epsilon = TEST_EPS)
        }
    }
}
