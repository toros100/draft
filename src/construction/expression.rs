// other object variants are defined in the object module
// (and value variants in the value module)
// but this one holds both object and value for expressions
//
// i did it like this because i anticipate writing a parser for expressions

use crate::construction::eval::{Eval, EvalCtx, EvalError};
use crate::construction::object::ObjectId;
use crate::construction::value::PointVal;
use std::ops::Add;
use std::ops::Mul;
use thiserror::Error;

// want to also have expression for curve length
// but i also want points on curves to produce sub-curves like in seamly
// and then i want to be able to also measure these, and not just the "first-class" curve
//
// i.e.: curve from a to b
// point c placed on curve at 100mm from start (or by intersecting curve and line?)
// -> implicit curve from a to c
//
// will also need this making pattern pieces by contour/path
// points dont accidentally end up on curves i guess, they are all special
// point variants that could know about their curve
// so a path could be "collapsed" with that?
// path (a, curve_a_b, c)
// c is ON curve_a_b, contour detects this?

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
    LineAngle(ObjectId, ObjectId),
    Dist(ObjectId, ObjectId),
    // could use ObjectId and put subexpressions into the arena instead of Box<ExpressionObj> for recursion
    // i guess that would be better for peak performance, could deduplicate expressions too
    Mul(Box<ExpressionObj>, Box<ExpressionObj>),
    Add(Box<ExpressionObj>, Box<ExpressionObj>),
    // TODO:
    // division (at least for scalars)
    // curve length including subcurves (oof)
    // further functions? trig? min/max? exponential?
    // small DLS + parser for input
    // what does seamly have? i think it even has conditionals?
}

impl ExpressionObj {
    pub fn type_check(&self) -> Result<(), ExpressionError> {
        todo!()
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
        }
    }
}
