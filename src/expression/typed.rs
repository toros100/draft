use super::{Angle, CurveId, Expression, Length, PointId, Scalar};
use std::ops::{Add, Mul};

// idea: wrapper around expressions that contains type-level info about what type these will
// evaluate to, with no way to construct them aside from these helper functions
//
// (dependencies, i.e. PointId/CurveId occuring in expressions, clearly need to be checked
// separately and require context)
//
// TODO: TryFrom<ExpressionObj> for LengthExpression/AngleExpression/ScalarExpression?

// maybe accumulate dependencies while building?

#[derive(Debug, Clone)]
pub struct LengthExpression {
    expr: Expression,
    is_const: bool,
}

#[derive(Debug, Clone)]
pub struct AngleExpression {
    expr: Expression,
    is_const: bool,
}

#[derive(Debug, Clone)]
pub struct ScalarExpression {
    expr: Expression,
    is_const: bool,
}

impl From<Length> for LengthExpression {
    fn from(value: Length) -> Self {
        LengthExpression {
            expr: Expression::Length(value),
            is_const: true,
        }
    }
}
impl From<Angle> for AngleExpression {
    fn from(value: Angle) -> Self {
        AngleExpression {
            expr: Expression::Angle(value),
            is_const: true,
        }
    }
}
impl From<Scalar> for ScalarExpression {
    fn from(value: Scalar) -> Self {
        ScalarExpression {
            expr: Expression::Scalar(value),
            is_const: true,
        }
    }
}

impl LengthExpression {
    pub fn inner(&self) -> &Expression {
        &self.expr
    }
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

impl AngleExpression {
    pub fn inner(&self) -> &Expression {
        &self.expr
    }
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

impl ScalarExpression {
    pub fn inner(&self) -> &Expression {
        &self.expr
    }
    pub fn is_const(&self) -> bool {
        self.is_const
    }
}

impl From<LengthExpression> for Expression {
    fn from(value: LengthExpression) -> Self {
        value.expr
    }
}
impl From<AngleExpression> for Expression {
    fn from(value: AngleExpression) -> Self {
        value.expr
    }
}
impl From<ScalarExpression> for Expression {
    fn from(value: ScalarExpression) -> Self {
        value.expr
    }
}

pub fn angle(v: f64) -> AngleExpression {
    AngleExpression {
        expr: Expression::Angle(super::Angle(v)),
        is_const: true,
    }
}

pub fn length(v: f64) -> LengthExpression {
    LengthExpression {
        expr: Expression::Length(super::Length(v)),
        is_const: true,
    }
}

pub fn scalar(v: f64) -> ScalarExpression {
    ScalarExpression {
        expr: Expression::Scalar(super::Scalar(v)),
        is_const: true,
    }
}

pub fn dist_between(from: PointId, to: PointId) -> LengthExpression {
    LengthExpression {
        expr: Expression::Dist(from, to),
        is_const: false,
    }
}

pub fn line_angle(from: PointId, to: PointId) -> AngleExpression {
    AngleExpression {
        expr: Expression::LineAngle(from, to),
        is_const: false,
    }
}

pub fn curve_length(c: CurveId) -> LengthExpression {
    LengthExpression {
        expr: Expression::CurveLength(c),
        is_const: false,
    }
}

impl Mul<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression {
            expr: Expression::Mul(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Mul<ScalarExpression> for LengthExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        LengthExpression {
            expr: Expression::Mul(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Mul<LengthExpression> for ScalarExpression {
    type Output = LengthExpression;
    fn mul(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression {
            expr: Expression::Mul(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Mul<ScalarExpression> for AngleExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: ScalarExpression) -> Self::Output {
        AngleExpression {
            expr: Expression::Mul(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Mul<AngleExpression> for ScalarExpression {
    type Output = AngleExpression;
    fn mul(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression {
            expr: Expression::Mul(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Add<ScalarExpression> for ScalarExpression {
    type Output = ScalarExpression;
    fn add(self, rhs: ScalarExpression) -> Self::Output {
        ScalarExpression {
            expr: Expression::Add(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Add<LengthExpression> for LengthExpression {
    type Output = LengthExpression;
    fn add(self, rhs: LengthExpression) -> Self::Output {
        LengthExpression {
            expr: Expression::Add(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
    }
}

impl Add<AngleExpression> for AngleExpression {
    type Output = AngleExpression;
    fn add(self, rhs: AngleExpression) -> Self::Output {
        AngleExpression {
            expr: Expression::Add(self.expr.into(), rhs.expr.into()),
            is_const: self.is_const && rhs.is_const,
        }
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
