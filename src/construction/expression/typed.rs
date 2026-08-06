use super::*;

// idea: wrapper around expressions that contains type-level info about what type these will
// evaluate to, with no way to construct them aside from these helper functions
//
// (dependencies, i.e. PointId/CurveId occuring in expressions, clearly needs to be checked
// separately and requires context)
//
// TODO: TryFrom<ExpressionObj> for LengthExpression/AngleExpression/ScalarExpression?

// maybe accumulate dependencies while building?

pub struct LengthExpression(ExpressionObj);
pub struct AngleExpression(ExpressionObj);
pub struct ScalarExpression(ExpressionObj);

impl From<LengthExpression> for ExpressionObj {
    fn from(value: LengthExpression) -> Self {
        value.0
    }
}
impl From<AngleExpression> for ExpressionObj {
    fn from(value: AngleExpression) -> Self {
        value.0
    }
}
impl From<ScalarExpression> for ExpressionObj {
    fn from(value: ScalarExpression) -> Self {
        value.0
    }
}

pub fn angle(v: f64) -> AngleExpression {
    AngleExpression(ExpressionObj::Angle(v))
}

pub fn length(v: f64) -> LengthExpression {
    LengthExpression(ExpressionObj::Length(v))
}

pub fn scalar(v: f64) -> ScalarExpression {
    ScalarExpression(ExpressionObj::Scalar(v))
}

pub fn dist_between(from: PointId, to: PointId) -> LengthExpression {
    LengthExpression(ExpressionObj::Dist(from, to))
}

pub fn line_angle(from: PointId, to: PointId) -> AngleExpression {
    AngleExpression(ExpressionObj::LineAngle(from, to))
}

pub fn curve_length(c: CurveId) -> LengthExpression {
    LengthExpression(ExpressionObj::CurveLength(c))
}

impl Mul<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression(ExpressionObj::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<ScalarExpression> for LengthExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        LengthExpression(ExpressionObj::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<LengthExpression> for ScalarExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression(ExpressionObj::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<ScalarExpression> for AngleExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        AngleExpression(ExpressionObj::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<AngleExpression> for ScalarExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression(ExpressionObj::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Add<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn add(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression(ExpressionObj::Add(self.0.into(), rhs.0.into()))
    }
}

impl Add<LengthExpression> for LengthExpression {
    type Output = LengthExpression;
    fn add(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression(ExpressionObj::Add(self.0.into(), rhs.0.into()))
    }
}

impl Add<AngleExpression> for AngleExpression {
    type Output = AngleExpression;
    fn add(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression(ExpressionObj::Add(self.0.into(), rhs.0.into()))
    }
}

// WARN: this is pretty cool syntactic sugar for raws treated as scalar expressions, but has some
// subtle effects w.r.t. constant folding

impl Mul<f64> for LengthExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: f64) -> Self::Output {
        self * scalar(rhs)
    }
}

impl Mul<LengthExpression> for f64 {
    type Output = LengthExpression;
    fn mul(self, rhs: LengthExpression) -> Self::Output {
        scalar(self) * rhs
    }
}

impl Mul<f64> for AngleExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: f64) -> Self::Output {
        self * scalar(rhs)
    }
}

impl Mul<AngleExpression> for f64 {
    type Output = AngleExpression;
    fn mul(self, rhs: AngleExpression) -> Self::Output {
        scalar(self) * rhs
    }
}

impl Mul<f64> for ScalarExpression {
    type Output = ScalarExpression;
    fn mul(self, rhs: f64) -> Self::Output {
        self * scalar(rhs)
    }
}

impl Mul<ScalarExpression> for f64 {
    type Output = ScalarExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        scalar(self) * rhs
    }
}

impl Add<f64> for ScalarExpression {
    type Output = ScalarExpression;
    fn add(self, rhs: f64) -> Self::Output {
        self + scalar(rhs)
    }
}

impl Add<ScalarExpression> for f64 {
    type Output = ScalarExpression;
    fn add(self, rhs: ScalarExpression) -> Self::Output {
        scalar(self) + rhs
    }
}
