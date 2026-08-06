// other object variants are defined in the object module
// (and value variants in the value module)
// but this one holds both object and value for expressions
//
// i did it like this because i anticipate writing a parser for expressions
//

mod typed;
pub use typed::*;

use crate::construction::eval::{Eval, EvalCtx, EvalError};
use crate::construction::object::{CurveId, ObjectId, PointId};
use crate::construction::value::{CurveVal, PointVal};
use crate::geom;
use std::ops::Add;
use std::ops::Mul;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExpressionError {
    #[error("unexpected operand type")]
    UnexpectedOperandType,
    #[error("unexpected result type")]
    UnexpectedValueType,
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

impl ExpressionObj {
    pub fn type_check(&self) -> Result<(), ExpressionError> {
        todo!()
    }

    pub fn push_dependencies(&self, dep: &mut Vec<ObjectId>) {
        match self {
            ExpressionObj::Mul(a, b) | ExpressionObj::Add(a, b) => {
                a.push_dependencies(dep);
                b.push_dependencies(dep);
            }
            ExpressionObj::Dist(a, b) | ExpressionObj::LineAngle(a, b) => {
                dep.extend::<[ObjectId; _]>([(*a).into(), (*b).into()])
            }
            ExpressionObj::CurveLength(c) => dep.push((*c).into()),
            ExpressionObj::Length(_) | ExpressionObj::Scalar(_) | ExpressionObj::Angle(_) => {}
        }
    }
}

#[derive(Clone, Copy)]
pub enum ExpressionVal {
    Length(f64),
    Angle(f64),
    Scalar(f64),
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

impl Eval for ExpressionObj {
    type Output = ExpressionVal;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError> {
        match self {
            ExpressionObj::Length(f) => Ok(ExpressionVal::Length(*f)),
            ExpressionObj::Angle(f) => Ok(ExpressionVal::Angle(*f)),
            ExpressionObj::Scalar(f) => Ok(ExpressionVal::Scalar(*f)),
            ExpressionObj::Add(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                Ok(a.try_add(b)?)
            }
            ExpressionObj::Mul(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                Ok(a.try_mul(b)?)
            }
            ExpressionObj::Dist(a, b) => {
                let p = ctx.try_get_as::<&PointVal>(a)?;
                let q = ctx.try_get_as::<&PointVal>(b)?;
                Ok(ExpressionVal::Length(p.pos.dist(q.pos)))
            }
            ExpressionObj::LineAngle(a, b) => {
                let p = ctx.try_get_as::<&PointVal>(a)?;
                let q = ctx.try_get_as::<&PointVal>(b)?;

                Ok(ExpressionVal::Angle(q.pos.angle(p.pos)))
            }
            ExpressionObj::CurveLength(a) => {
                let c = ctx.try_get_as::<&CurveVal>(a)?;
                let l = geom::cubic_bezier_length(c.from, c.control_1, c.control_2, c.to);
                Ok(ExpressionVal::Length(l))
            }
        }
    }
}
