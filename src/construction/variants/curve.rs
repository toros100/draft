use super::*;
use crate::core::*;
use crate::geom;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveId(pub(crate) usize);

impl From<usize> for CurveId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<CurveId> for ObjectId {
    fn from(value: CurveId) -> Self {
        Self::Curve(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CurveObj {
    pub from: PointId,
    pub to: PointId,
    pub from_control: CurveControlId,
    pub to_control: CurveControlId,
}

impl From<CurveObj> for Object {
    fn from(value: CurveObj) -> Self {
        Object::Curve(value)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CurveVal {
    pub curve: geom::CubicBezier,
}

impl From<CurveVal> for Value {
    fn from(value: CurveVal) -> Self {
        Value::Curve(value)
    }
}

impl Variant<Object> for CurveObj {
    type Val = CurveVal;
    type Id = CurveId;
    type EvalError = EvalError;
    fn dependencies(&self, dst: &mut impl Extend<<Object as SumObject>::Id>) {
        dst.extend([
            self.from.into(),
            self.to.into(),
            self.from_control.into(),
            self.to_control.into(),
        ]);
    }

    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<Object>) -> Result<(), Self::EvalError> {
        let from = ctx
            .get_cached_as::<PointObj>(self.from)
            .ok_or(EvalError::UnknownDependency)?;
        let to = ctx
            .get_cached_as::<PointObj>(self.to)
            .ok_or(EvalError::UnknownDependency)?;
        let control_1 = ctx
            .get_cached_as::<CurveControlObj>(self.from_control)
            .ok_or(EvalError::UnknownDependency)?;
        let control_2 = ctx
            .get_cached_as::<CurveControlObj>(self.to_control)
            .ok_or(EvalError::UnknownDependency)?;

        dst.curve = geom::curve(from.pos, control_1.pos, control_2.pos, to.pos);
        Ok(())
    }
}

impl Case<ObjectId> for CurveId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::Curve(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::Curve(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Object> for CurveObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Curve(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Curve(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for CurveVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Curve(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Curve(inner) => Some(inner),
            _ => None,
        }
    }
}
