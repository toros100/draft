use crate::construction::{CurveId, EvalCtx, EvalError, LengthVariableId, ObjectId, PointId};
use std::error::Error;
use std::ops::Add;
use std::ops::Mul;
use thiserror::Error;

mod typed;
pub use typed::*;

// need a way to resolve identifiers (string to ObjectId and reverse)
pub struct Symbols {}

pub trait Stringify {
    fn stringify(&self, sym: &Symbols) -> String;
}

#[derive(thiserror::Error, Debug)]
#[error("failed to parse: {}", .0)]
pub struct ParseError(Box<dyn Error>);

pub trait Parse: Sized {
    fn parse(input: impl AsRef<str>, sym: &Symbols) -> Result<Self, ParseError>;
}

// TODO: actual impl for Stringify/Parse
// probably extract expression module to somewhere else
impl Stringify for Expression {
    fn stringify(&self, _: &Symbols) -> String {
        match self {
            Expression::Length(l) => l.0.to_string(),
            Expression::Angle(l) => l.0.to_string(),
            _ => unimplemented!(),
        }
    }
}

impl Parse for LengthExpression {
    fn parse(input: impl AsRef<str>, _: &Symbols) -> Result<Self, ParseError> {
        Ok(length(
            input
                .as_ref()
                .parse()
                .map_err(|e| ParseError(Box::new(e)))?,
        ))
    }
}

impl Parse for AngleExpression {
    fn parse(input: impl AsRef<str>, _: &Symbols) -> Result<Self, ParseError> {
        Ok(angle(
            input
                .as_ref()
                .parse()
                .map_err(|e| ParseError(Box::new(e)))?,
        ))
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Length(Length),
    Angle(Angle),
    Scalar(Scalar),
    LengthVar(LengthVariableId),
    LineAngle(PointId, PointId),
    Dist(PointId, PointId),
    CurveLength(CurveId),
    Mul(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn type_check(&self) -> Result<(), ExpressionError> {
        todo!()
    }

    pub fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        match self {
            Expression::Mul(a, b) | Expression::Add(a, b) => {
                a.dependencies(dst);
                b.dependencies(dst);
            }
            Expression::Dist(a, b) | Expression::LineAngle(a, b) => {
                dst.extend::<[ObjectId; _]>([(*a).into(), (*b).into()])
            }
            Expression::CurveLength(c) => dst.extend([(*c).into()]),
            Expression::Length(_) | Expression::Scalar(_) | Expression::Angle(_) => {}
            Expression::LengthVar(v) => {
                dst.extend([(*v).into()]);
            }
        }
    }

    pub fn eval_expr(&self, ctx: &EvalCtx) -> Result<ExpressionVal, EvalError> {
        match self {
            Expression::Length(f) => Ok(ExpressionVal::Length(*f)),
            Expression::Angle(f) => Ok(ExpressionVal::Angle(*f)),
            Expression::Scalar(f) => Ok(ExpressionVal::Scalar(*f)),
            Expression::Add(a, b) => {
                let a = a.eval_expr(ctx)?;
                let b = b.eval_expr(ctx)?;
                Ok(a.try_add(b)?)
            }
            Expression::Mul(a, b) => {
                let a = a.eval_expr(ctx)?;
                let b = b.eval_expr(ctx)?;
                Ok(a.try_mul(b)?)
            }
            Expression::Dist(a, b) => {
                let p = ctx.get_point_position(*a)?;
                let q = ctx.get_point_position(*b)?;
                Ok(ExpressionVal::Length(Length(p.dist(q))))
            }
            Expression::LineAngle(a, b) => {
                let p = ctx.get_point_position(*a)?;
                let q = ctx.get_point_position(*b)?;
                Ok(ExpressionVal::Angle(Angle(p.angle(q))))
            }
            Expression::CurveLength(a) => {
                let c = ctx.get_curve(*a)?;
                Ok(ExpressionVal::Length(Length(c.curve.approx_length())))
            }
            Expression::LengthVar(id) => {
                let c = ctx.get_length_var(*id)?;
                Ok(c.into())
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("unexpected operand type")]
    UnexpectedOperandType,
    #[error("unexpected result type")]
    UnexpectedResultType,
}

impl From<ExpressionError> for EvalError {
    fn from(value: ExpressionError) -> Self {
        EvalError::ExpressionError(value)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Length(f64);

#[derive(Debug, Clone, Copy, Default)]
pub struct Angle(f64);

#[derive(Debug, Clone, Copy, Default)]
pub struct Scalar(f64);

impl From<Length> for f64 {
    fn from(value: Length) -> Self {
        value.0
    }
}
impl From<Angle> for f64 {
    fn from(value: Angle) -> Self {
        value.0
    }
}

impl Add<Length> for Length {
    type Output = Length;
    fn add(self, rhs: Length) -> Self::Output {
        Length(self.0 + rhs.0)
    }
}

impl Mul<Scalar> for Length {
    type Output = Length;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Length(self.0 * rhs.0)
    }
}
impl Mul<Length> for Scalar {
    type Output = Length;
    fn mul(self, rhs: Length) -> Self::Output {
        Length(self.0 * rhs.0)
    }
}

impl Add<Scalar> for Scalar {
    type Output = Scalar;
    fn add(self, rhs: Scalar) -> Self::Output {
        Scalar(self.0 + rhs.0)
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Scalar(self.0 * rhs.0)
    }
}

impl Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, rhs: Angle) -> Self::Output {
        Angle(self.0 + rhs.0)
    }
}

impl Mul<Scalar> for Angle {
    type Output = Angle;
    fn mul(self, rhs: Scalar) -> Self::Output {
        Angle(self.0 * rhs.0)
    }
}

impl Mul<Angle> for Scalar {
    type Output = Angle;
    fn mul(self, rhs: Angle) -> Self::Output {
        Angle(self.0 * rhs.0)
    }
}

#[derive(Debug, Clone, Copy, derive_more::From)]
pub enum ExpressionVal {
    Length(Length),
    Angle(Angle),
    Scalar(Scalar),
}

impl ExpressionVal {
    pub fn try_as_length(self) -> Result<Length, ExpressionError> {
        match self {
            ExpressionVal::Length(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedResultType),
        }
    }
    pub fn try_as_angle(self) -> Result<Angle, ExpressionError> {
        match self {
            ExpressionVal::Angle(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedResultType),
        }
    }
    pub fn try_as_scalar(self) -> Result<Scalar, ExpressionError> {
        match self {
            ExpressionVal::Scalar(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedResultType),
        }
    }

    pub fn try_add(self, other: ExpressionVal) -> Result<ExpressionVal, ExpressionError> {
        match (self, other) {
            (ExpressionVal::Length(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.add(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.add(b)))
            }
            (ExpressionVal::Scalar(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Scalar(a.add(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }

    pub fn try_mul(self, other: ExpressionVal) -> Result<ExpressionVal, ExpressionError> {
        match (self, other) {
            (ExpressionVal::Scalar(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Scalar(a.mul(b)))
            }
            (ExpressionVal::Length(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Length(a.mul(b)))
            }
            (ExpressionVal::Scalar(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.mul(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Angle(a.mul(b)))
            }
            (ExpressionVal::Scalar(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.mul(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }
}
