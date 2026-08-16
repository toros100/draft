use super::*;
use crate::arena::Arena;
use crate::core::*;
use std::ops::Add;
use std::ops::Mul;
use thiserror::Error;

mod typed;
pub use typed::*;

pub trait Stringify {
    fn stringify(&self, arena: &Arena<Object>) -> String;
}

pub trait Parse {
    fn parse(input: impl AsRef<str>, arena: &Arena<Object>) -> Self;
}

// TODO: actual impl for Stringify/Parse
// probably extract expression module to somewhere else
impl Stringify for Expression {
    fn stringify(&self, _: &Arena<Object>) -> String {
        match self {
            Expression::Length(l) => l.to_string(),
            Expression::Angle(l) => l.to_string(),
            _ => unimplemented!(),
        }
    }
}

impl Parse for LengthExpression {
    fn parse(input: impl AsRef<str>, _: &Arena<Object>) -> Self {
        length(input.as_ref().parse().unwrap())
    }
}

impl Parse for AngleExpression {
    fn parse(input: impl AsRef<str>, _: &Arena<Object>) -> Self {
        angle(input.as_ref().parse().unwrap())
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Length(f64),
    Angle(f64),
    Scalar(f64),
    LengthVar(LengthVariableId),
    AngleVar(AngleVariableId),
    ScalarVar(ScalarVariableId),
    LineAngle(PointId, PointId),
    Dist(PointId, PointId),
    CurveLength(CurveId),
    Mul(Box<Expression>, Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    // TODO:
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
            Expression::AngleVar(v) => {
                dst.extend([v.inner().into()]);
            }
            Expression::LengthVar(v) => {
                dst.extend([v.inner().into()]);
            }
            Expression::ScalarVar(v) => {
                dst.extend([v.inner().into()]);
            }
        }
    }

    pub fn eval_expr(&self, ctx: &impl EvalCtx<Object>) -> Result<ExpressionVal, EvalError> {
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
                let p = ctx
                    .get_cached_as::<PointObj>(*a)
                    .ok_or(EvalError::UnexpectedType)?;
                let q = ctx
                    .get_cached_as::<PointObj>(*b)
                    .ok_or(EvalError::UnknownDependency)?;
                Ok(ExpressionVal::Length(p.pos.dist(q.pos)))
            }
            Expression::LineAngle(a, b) => {
                let p = ctx
                    .get_cached_as::<PointObj>(*a)
                    .ok_or(EvalError::UnknownDependency)?;
                let q = ctx
                    .get_cached_as::<PointObj>(*b)
                    .ok_or(EvalError::UnknownDependency)?;

                Ok(ExpressionVal::Angle(q.pos.angle(p.pos)))
            }
            Expression::CurveLength(a) => {
                let c = ctx
                    .get_cached_as::<CurveObj>(*a)
                    .ok_or(EvalError::UnknownDependency)?;
                Ok(ExpressionVal::Length(c.curve.approx_length()))
            }
            Expression::AngleVar(v) => {
                let c = ctx
                    .get_cached_as::<VariableObj>(v.inner())
                    .ok_or(EvalError::UnknownDependency)?
                    .val
                    .try_as_angle()?;
                Ok(ExpressionVal::Angle(c))
            }
            Expression::LengthVar(v) => {
                let c = ctx
                    .get_cached_as::<VariableObj>(v.inner())
                    .ok_or(EvalError::UnknownDependency)?
                    .val
                    .try_as_length()?;
                Ok(ExpressionVal::Length(c))
            }
            Expression::ScalarVar(v) => {
                let c = ctx
                    .get_cached_as::<VariableObj>(v.inner())
                    .ok_or(EvalError::UnknownDependency)?
                    .val
                    .try_as_scalar()?;
                Ok(ExpressionVal::Scalar(c))
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("unexpected operand type")]
    UnexpectedOperandType,
    #[error("unexpected result type")]
    UnexpectedValueType,
}

impl From<ExpressionError> for EvalError {
    fn from(value: ExpressionError) -> Self {
        EvalError::ExpressionError(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExpressionVal {
    Length(f64),
    Angle(f64),
    Scalar(f64),
}

impl Default for ExpressionVal {
    fn default() -> Self {
        Self::Length(0.)
    }
}

impl ExpressionVal {
    pub fn try_as_length(self) -> Result<f64, ExpressionError> {
        match self {
            ExpressionVal::Length(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedValueType),
        }
    }
    pub fn try_as_angle(self) -> Result<f64, ExpressionError> {
        match self {
            ExpressionVal::Angle(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedValueType),
        }
    }
    pub fn try_as_scalar(self) -> Result<f64, ExpressionError> {
        match self {
            ExpressionVal::Scalar(l) => Ok(l),
            _ => Err(ExpressionError::UnexpectedValueType),
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
            (ExpressionVal::Angle(a), ExpressionVal::Scalar(b))
            | (ExpressionVal::Scalar(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.add(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }

    pub fn try_mul(self, other: ExpressionVal) -> Result<ExpressionVal, ExpressionError> {
        match (self, other) {
            (ExpressionVal::Scalar(a), ExpressionVal::Scalar(b)) => {
                Ok(ExpressionVal::Scalar(a.mul(b)))
            }
            (ExpressionVal::Length(a), ExpressionVal::Scalar(b))
            | (ExpressionVal::Scalar(a), ExpressionVal::Length(b)) => {
                Ok(ExpressionVal::Length(a.mul(b)))
            }
            (ExpressionVal::Angle(a), ExpressionVal::Scalar(b))
            | (ExpressionVal::Scalar(a), ExpressionVal::Angle(b)) => {
                Ok(ExpressionVal::Angle(a.mul(b)))
            }
            _ => Err(ExpressionError::UnexpectedOperandType),
        }
    }
}
