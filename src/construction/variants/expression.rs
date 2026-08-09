use super::*;
use crate::core::*;
use std::ops::Add;
use std::ops::Mul;
use thiserror::Error;

mod typed;
pub use typed::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ExpressionId(pub(crate) usize);

impl From<ExpressionId> for ObjectId {
    fn from(value: ExpressionId) -> Self {
        Self::Expression(value)
    }
}

impl From<usize> for ExpressionId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug)]
pub enum ExpressionObj {
    Length(f64),
    Angle(f64),
    Scalar(f64),
    LineAngle(PointId, PointId),
    Dist(PointId, PointId),
    CurveLength(CurveId),
    // could use ObjectId and put subexpressions into the arena instead of Box<ExpressionObj> for recursion
    // i guess that would be better for peak performance, could deduplicate expressions too
    // (would be good if for example n expressions contain CurveLength(id) with the same id,
    // currently this would calculate the curve length n times)
    Mul(Box<ExpressionObj>, Box<ExpressionObj>),
    Add(Box<ExpressionObj>, Box<ExpressionObj>),
    // TODO:
    // sub, div, unary negative
    // unit variants of leaf expressions? e.g. instead of Length(f64), do something like Length(ConstLength)
    // and ConstLength { Mm(f64), Cm(f64) } etc (rad/deg angles)
    // further functions? trig? min/max? exponential?
    // small DLS + parser for input
    // what does seamly have? i think it even has conditionals?
}

impl From<ExpressionObj> for Object {
    fn from(value: ExpressionObj) -> Self {
        Object::Expression(value)
    }
}

impl ExpressionObj {
    pub fn type_check(&self) -> Result<(), ExpressionError> {
        todo!()
    }

    pub fn walk_dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        match self {
            ExpressionObj::Mul(a, b) | ExpressionObj::Add(a, b) => {
                a.walk_dependencies(dst);
                b.walk_dependencies(dst);
            }
            ExpressionObj::Dist(a, b) | ExpressionObj::LineAngle(a, b) => {
                dst.extend::<[ObjectId; _]>([(*a).into(), (*b).into()])
            }
            ExpressionObj::CurveLength(c) => dst.extend([(*c).into()]),
            ExpressionObj::Length(_) | ExpressionObj::Scalar(_) | ExpressionObj::Angle(_) => {}
        }
    }

    fn eval_expr(&self, ctx: &impl EvalCtx<Object>) -> Result<ExpressionVal, EvalError> {
        match self {
            ExpressionObj::Length(f) => Ok(ExpressionVal::Length(*f)),
            ExpressionObj::Angle(f) => Ok(ExpressionVal::Angle(*f)),
            ExpressionObj::Scalar(f) => Ok(ExpressionVal::Scalar(*f)),
            ExpressionObj::Add(a, b) => {
                let a = a.eval_expr(ctx)?;
                let b = b.eval_expr(ctx)?;
                Ok(a.try_add(b)?)
            }
            ExpressionObj::Mul(a, b) => {
                let a = a.eval_expr(ctx)?;
                let b = b.eval_expr(ctx)?;
                Ok(a.try_mul(b)?)
            }
            ExpressionObj::Dist(a, b) => {
                let p = ctx
                    .get_cached_as::<PointObj>(*a)
                    .ok_or(EvalError::UnexpectedType)?;
                let q = ctx
                    .get_cached_as::<PointObj>(*b)
                    .ok_or(EvalError::UnknownDependency)?;
                Ok(ExpressionVal::Length(p.pos.dist(q.pos)))
            }
            ExpressionObj::LineAngle(a, b) => {
                let p = ctx
                    .get_cached_as::<PointObj>(*a)
                    .ok_or(EvalError::UnknownDependency)?;
                let q = ctx
                    .get_cached_as::<PointObj>(*b)
                    .ok_or(EvalError::UnknownDependency)?;

                Ok(ExpressionVal::Angle(q.pos.angle(p.pos)))
            }
            ExpressionObj::CurveLength(a) => {
                let c = ctx
                    .get_cached_as::<CurveObj>(*a)
                    .ok_or(EvalError::UnknownDependency)?;
                Ok(ExpressionVal::Length(c.curve.approx_length()))
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

impl From<ExpressionVal> for Value {
    fn from(value: ExpressionVal) -> Self {
        Self::Expression(value)
    }
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

impl Variant<Object> for ExpressionObj {
    type EvalError = EvalError;
    type Id = ExpressionId;
    type Val = ExpressionVal;
    fn dependencies(&self, dst: &mut impl Extend<<Object as SumObject>::Id>) {
        self.walk_dependencies(dst);
    }

    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<Object>) -> Result<(), Self::EvalError> {
        *dst = self.eval_expr(ctx)?;
        Ok(())
    }
}

impl Case<ObjectId> for ExpressionId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::Expression(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::Expression(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Object> for ExpressionObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Expression(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Expression(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for ExpressionVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Expression(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Expression(inner) => Some(inner),
            _ => None,
        }
    }
}
