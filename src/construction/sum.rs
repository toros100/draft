use super::variants::*;
use crate::core::*;
use std::{fmt::Debug, hash::Hash};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Point(PointId),
    Line(LineId),
    Curve(CurveId),
    CurveControl(CurveControlId),
    Expression(ExpressionId),
}

impl From<ObjectId> for usize {
    fn from(value: ObjectId) -> Self {
        match value {
            ObjectId::Point(inner) => inner.0,
            ObjectId::Line(inner) => inner.0,
            ObjectId::CurveControl(inner) => inner.0,
            ObjectId::Curve(inner) => inner.0,
            ObjectId::Expression(inner) => inner.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Object {
    // a point
    Point(PointObj),

    // line between two points
    Line(LineObj),

    // cubic bezier between two points
    Curve(CurveObj),

    // bezier control point
    // functionally similar to Point::DistAngle, but i did not want make it a "first-class point",
    // because i do not want stuff like curves or other points to connect to these synthetic control
    // points.
    // this is kind of a "leaf" point, opposite to PointObj::Root:
    // may depend on other points, but no other point may depend on it?
    // might still want to allow its measurements (dist/angle) to be used in expressions?
    CurveControl(CurveControlObj),

    Expression(ExpressionObj),
}

#[derive(Debug, Clone)]
// not copy because i anticipate future non-copy variants
pub enum Value {
    Point(PointVal),
    Line(LineVal),
    Curve(CurveVal),
    CurveControl(CurveControlVal),
    Expression(ExpressionVal),
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("unresolved dependency (order broken)")]
    UnresolvedDependency,
    #[error("unknown dependency")]
    UnknownDependency,
    #[error("unexpected value type")]
    UnexpectedType,
    #[error("expression error: {}", .0)]
    ExpressionError(ExpressionError),
}

impl SumObject for Object {
    type Id = ObjectId;
    type EvalError = EvalError;
    type Value = Value;
    fn eval_dispatch(
        &self,
        dst: &mut Option<Self::Value>,
        ctx: &impl EvalCtx<Self>,
    ) -> Result<(), Self::EvalError> {
        match self {
            Object::Point(inner) => inner.eval(project_or_default(dst), ctx),
            Object::Line(inner) => inner.eval(project_or_default(dst), ctx),
            Object::CurveControl(inner) => inner.eval(project_or_default(dst), ctx),
            Object::Curve(inner) => inner.eval(project_or_default(dst), ctx),
            Object::Expression(inner) => inner.eval(project_or_default(dst), ctx),
        }
    }
}

fn project_or_default<S, T>(opt: &mut Option<S>) -> &mut T
where
    T: Case<S> + Default,
{
    let none_or_wrong_variant = match opt.as_mut() {
        Some(s) => T::project_mut(s).is_none(),
        None => true,
    };

    if none_or_wrong_variant {
        _ = opt.insert(T::default().into());
    };

    opt.as_mut().unwrap().case_mut().unwrap()
}
