use std::fmt::Debug;

use crate::construction::eval::EvalError;
use crate::construction::expression::ExpressionVal;
use crate::geom::{CubicBezier, Point2};

#[derive(Clone)]
// not copy because i anticipate future non-copy variants
pub enum Value {
    Point(PointVal),
    Line(LineVal),
    Curve(CurveVal),
    CurveControl(CurveControlVal),
    Expression(ExpressionVal),
}

pub trait TryProject<From> {
    type Error: Debug;
    fn try_project(val: &From) -> Result<&Self, Self::Error>;
}

impl TryProject<Value> for PointVal {
    type Error = EvalError;
    fn try_project(val: &Value) -> Result<&Self, EvalError> {
        match val {
            Value::Point(v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}
impl TryProject<Value> for LineVal {
    type Error = EvalError;
    fn try_project(val: &Value) -> Result<&Self, EvalError> {
        match val {
            Value::Line(v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}

impl TryProject<Value> for CurveVal {
    type Error = EvalError;
    fn try_project(val: &Value) -> Result<&Self, EvalError> {
        match val {
            Value::Curve(v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}

impl TryProject<Value> for CurveControlVal {
    type Error = EvalError;
    fn try_project(val: &Value) -> Result<&Self, EvalError> {
        match val {
            Value::CurveControl(v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}

impl TryProject<Value> for ExpressionVal {
    type Error = EvalError;
    fn try_project(val: &Value) -> Result<&Self, EvalError> {
        match val {
            Value::Expression(v) => Ok(v),
            _ => Err(EvalError::UnexpectedType),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PointVal {
    pub pos: Point2,
}

impl From<PointVal> for Value {
    fn from(value: PointVal) -> Self {
        Value::Point(value)
    }
}

#[derive(Clone, Copy)]
pub struct LineVal {
    pub from: Point2,
    pub to: Point2,
}

#[derive(Clone, Copy)]
pub struct CurveVal {
    pub curve: CubicBezier,
}

#[derive(Clone, Copy)]
pub struct CurveControlVal {
    pub pos: Point2,
    pub parent: Point2,
}

impl<'a> TryFrom<&'a Value> for &'a PointVal {
    type Error = EvalError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Point(p) = value {
            Ok(p)
        } else {
            Err(EvalError::UnexpectedType)
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a CurveControlVal {
    type Error = EvalError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::CurveControl(c) = value {
            Ok(c)
        } else {
            Err(EvalError::UnexpectedType)
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a LineVal {
    type Error = EvalError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Line(l) = value {
            Ok(l)
        } else {
            Err(EvalError::UnexpectedType)
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a CurveVal {
    type Error = EvalError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Curve(l) = value {
            Ok(l)
        } else {
            Err(EvalError::UnexpectedType)
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a ExpressionVal {
    type Error = EvalError;
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        if let Value::Expression(e) = value {
            Ok(e)
        } else {
            Err(EvalError::UnexpectedType)
        }
    }
}
