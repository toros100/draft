use super::*;

// idea: wrapper around expressions that contains type-level info about what type these will
// evaluate to, with no way to construct them aside from these helper functions
//
// (dependencies, i.e. PointId/CurveId occuring in expressions, clearly need to be checked
// separately and require context)
//
// TODO: TryFrom<ExpressionObj> for LengthExpression/AngleExpression/ScalarExpression?

// maybe accumulate dependencies while building?

#[derive(Debug, Clone)]
pub struct LengthExpression(Expression);

#[derive(Debug, Clone)]
pub struct AngleExpression(Expression);

#[derive(Debug, Clone)]
pub struct ScalarExpression(Expression);

impl LengthExpression {
    pub fn inner(&self) -> &Expression {
        &self.0
    }
}

impl AngleExpression {
    pub fn inner(&self) -> &Expression {
        &self.0
    }
}

impl ScalarExpression {
    pub fn inner(&self) -> &Expression {
        &self.0
    }
}

impl From<LengthExpression> for Expression {
    fn from(value: LengthExpression) -> Self {
        value.0
    }
}
impl From<AngleExpression> for Expression {
    fn from(value: AngleExpression) -> Self {
        value.0
    }
}
impl From<ScalarExpression> for Expression {
    fn from(value: ScalarExpression) -> Self {
        value.0
    }
}

pub fn angle(v: f64) -> AngleExpression {
    AngleExpression(Expression::Angle(v))
}

pub fn length(v: f64) -> LengthExpression {
    LengthExpression(Expression::Length(v))
}

pub fn scalar(v: f64) -> ScalarExpression {
    ScalarExpression(Expression::Scalar(v))
}

pub fn dist_between(from: PointId, to: PointId) -> LengthExpression {
    LengthExpression(Expression::Dist(from, to))
}

pub fn line_angle(from: PointId, to: PointId) -> AngleExpression {
    AngleExpression(Expression::LineAngle(from, to))
}

pub fn curve_length(c: CurveId) -> LengthExpression {
    LengthExpression(Expression::CurveLength(c))
}

impl Mul<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression(Expression::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<ScalarExpression> for LengthExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        LengthExpression(Expression::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<LengthExpression> for ScalarExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression(Expression::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<ScalarExpression> for AngleExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        AngleExpression(Expression::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Mul<AngleExpression> for ScalarExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression(Expression::Mul(self.0.into(), rhs.0.into()))
    }
}

impl Add<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn add(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression(Expression::Add(self.0.into(), rhs.0.into()))
    }
}

impl Add<LengthExpression> for LengthExpression {
    type Output = LengthExpression;
    fn add(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression(Expression::Add(self.0.into(), rhs.0.into()))
    }
}

impl Add<AngleExpression> for AngleExpression {
    type Output = AngleExpression;
    fn add(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression(Expression::Add(self.0.into(), rhs.0.into()))
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
