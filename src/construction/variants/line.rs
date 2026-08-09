use super::*;
use crate::core::*;
use crate::geom::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LineId(pub(crate) usize);

impl From<LineId> for ObjectId {
    fn from(value: LineId) -> Self {
        Self::Line(value)
    }
}

impl From<usize> for LineId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineObj {
    pub from: PointId,
    pub to: PointId,
}

impl From<LineObj> for Object {
    fn from(value: LineObj) -> Self {
        Object::Line(value)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LineVal {
    pub from: Point2,
    pub to: Point2,
}

impl From<LineVal> for Value {
    fn from(value: LineVal) -> Self {
        Value::Line(value)
    }
}

impl Variant<Object> for LineObj {
    type Val = LineVal;
    type Id = LineId;
    type EvalError = EvalError;

    fn dependencies(&self, dst: &mut impl Extend<<Object as SumObject>::Id>) {
        dst.extend([self.from.into(), self.to.into()]);
    }

    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<Object>) -> Result<(), Self::EvalError> {
        let to = ctx
            .get_cached_as::<PointObj>(self.to)
            .ok_or(EvalError::UnknownDependency)?;

        let from = ctx
            .get_cached_as::<PointObj>(self.from)
            .ok_or(EvalError::UnknownDependency)?;

        dst.to = to.pos;
        dst.from = from.pos;
        Ok(())
    }
}

impl Case<ObjectId> for LineId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::Line(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::Line(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Object> for LineObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Line(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Line(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for LineVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Line(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Line(inner) => Some(inner),
            _ => None,
        }
    }
}
