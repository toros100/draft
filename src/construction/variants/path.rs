use crate::{
    construction::{CurveId, EvalError, Object, ObjectId, PointFree, PointFreeId, Value},
    core::*,
    geom::{CubicBezier, Point2},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathId(pub(crate) usize);

impl From<usize> for PathId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathSegment {
    Point(PointFreeId),
    Curve(CurveId), // TODO: allow reversing curves
}

#[derive(Debug, Clone)]
pub struct PathObj {
    // need curves too eventually
    pub parts: Vec<PathSegment>,
}

#[derive(Debug, Clone)]
pub enum PathSementVal {
    Point(Point2),
    Curve(CubicBezier),
}

#[derive(Debug, Default, Clone)]
pub struct PathVal {
    pub points: Vec<PathSementVal>,
}

impl Case<ObjectId> for PathId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::Path(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::Path(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Object> for PathObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Path(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Path(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for PathVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Path(inner) => Some(inner),
            _ => None,
        }
    }

    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Path(inner) => Some(inner),
            _ => None,
        }
    }
}

impl VariantOld<Object> for PathObj {
    type Id = PathId;
    type Val = PathVal;
    type EvalError = EvalError;
    fn eval_old(
        &self,
        dst: &mut Self::Val,
        ctx: &impl crate::core::EvalCtxOld<Object>,
    ) -> Result<(), Self::EvalError> {
        dst.points.clear();

        for p in self.parts.iter() {
            match p {
                PathSegment::Point(p) => dst.points.push(PathSementVal::Point(
                    ctx.get_cached_as::<PointFree>(*p).unwrap().pos,
                )),
                _ => unimplemented!(),
            }
        }

        // TODO: add curves and implement partial curve paths
        // e.g. curve followed by a point on the curve only takes the curve up until that point
        // also curve rerversal
        // cf. seamly

        Ok(())
    }
    fn dependencies_old(&self, dst: &mut impl Extend<<Object as crate::core::SumObject>::Id>) {
        dst.extend(self.parts.iter().map(|p| match p {
            PathSegment::Point(p) => (*p).into(),
            PathSegment::Curve(c) => (*c).into(),
        }));
    }
}
