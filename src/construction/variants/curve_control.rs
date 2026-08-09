use super::*;
use crate::core::*;
use crate::geom::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveControlId(pub(crate) usize);

impl From<CurveControlId> for ObjectId {
    fn from(value: CurveControlId) -> Self {
        Self::CurveControl(value)
    }
}

impl From<usize> for CurveControlId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CurveControlObj {
    pub parent: PointId,
    pub off: Polar,
}

impl From<CurveControlObj> for Object {
    fn from(value: CurveControlObj) -> Self {
        Object::CurveControl(value)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CurveControlVal {
    pub pos: Point2,
    pub parent: Point2,
}

impl From<CurveControlVal> for Value {
    fn from(value: CurveControlVal) -> Self {
        Self::CurveControl(value)
    }
}

impl Variant<Object> for CurveControlObj {
    type EvalError = EvalError;
    type Id = CurveControlId;
    type Val = CurveControlVal;
    fn dependencies(&self, dst: &mut impl Extend<<Object as SumObject>::Id>) {
        dst.extend([self.parent.into()]);
    }
    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<Object>) -> Result<(), Self::EvalError> {
        let parent = ctx
            .get_cached_as::<PointObj>(self.parent)
            .ok_or(EvalError::UnknownDependency)?;

        dst.parent = parent.pos;
        dst.pos = parent.pos + self.off;

        Ok(())
    }
}

impl Case<ObjectId> for CurveControlId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Object> for CurveControlObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for CurveControlVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::CurveControl(inner) => Some(inner),
            _ => None,
        }
    }
}
