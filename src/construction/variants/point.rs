use crate::construction::variants::expression::*;
use crate::construction::*;
use crate::core::*;
use crate::geom::Point2;
use std::iter::Extend;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointId(pub(crate) usize);

impl From<usize> for PointId {
    fn from(value: usize) -> Self {
        PointId(value)
    }
}

impl From<PointId> for ObjectId {
    fn from(value: PointId) -> Self {
        Self::Point(value)
    }
}

#[derive(Clone, Debug)]
pub enum PointObj {
    // point with absolute position
    Root {
        pos: Point2,
    },

    // point at distance and angle from another point
    DistAngle {
        parent: PointId,        // must refer to Object::Point in arena
        dist: LengthExpression, // must refer to Object::Expression
        angle: AngleExpression, // same
    },

    // point on line between two points
    // deliberately not referring to Object::Line, which are "drawn" lines
    OnLine {
        from: PointId, // must refer to Object::Point
        to: PointId,   // ...
        dist: LengthExpression,
    },

    // point on a curve
    OnCurve {
        curve: CurveId,
        dist: LengthExpression,
    },
}

impl From<PointObj> for Object {
    fn from(value: PointObj) -> Self {
        Object::Point(value)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointVal {
    pub pos: Point2,
}

impl From<PointVal> for Value {
    fn from(value: PointVal) -> Self {
        Value::Point(value)
    }
}

impl Variant<Object> for PointObj {
    type EvalError = EvalError;
    type Id = PointId;
    type Val = PointVal;
    fn dependencies(&self, dst: &mut impl Extend<<Object as SumObject>::Id>) {
        match self {
            PointObj::OnLine { from, to, dist } => {
                dist.inner().dependencies(dst);
                dst.extend::<[ObjectId; _]>([(*from).into(), (*to).into()])
            }
            PointObj::DistAngle {
                parent,
                dist,
                angle,
            } => {
                dst.extend::<[ObjectId; _]>([(*parent).into()]);
                dist.inner().dependencies(dst);
                angle.inner().dependencies(dst);
            }
            PointObj::OnCurve { curve, dist } => {
                dist.inner().dependencies(dst);
                dst.extend::<[ObjectId; _]>([(*curve).into()])
            }
            // writing this case out explicitly rather than using a wildcard so it will break when i
            // add variants to this enum rather than produce garbage
            PointObj::Root { .. } => {}
        }
    }
    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<Object>) -> Result<(), Self::EvalError> {
        match self {
            PointObj::Root { pos: p } => dst.pos = *p,
            PointObj::DistAngle {
                parent,
                dist,
                angle,
            } => {
                let d = dist.inner().eval_expr(ctx)?.try_as_length()?;
                let a = angle.inner().eval_expr(ctx)?.try_as_angle()?;

                let off = Polar::new(d, a);
                let p = ctx
                    .get_cached_as::<PointObj>(*parent)
                    .ok_or(EvalError::UnknownDependency)?;

                dst.pos = p.pos + off;
            }
            PointObj::OnLine { from, to, dist } => {
                let from_pos = ctx
                    .get_cached_as::<PointObj>(*from)
                    .ok_or(EvalError::UnknownDependency)?;
                let to_pos = ctx
                    .get_cached_as::<PointObj>(*to)
                    .ok_or(EvalError::UnknownDependency)?;

                let dist = dist.inner().eval_expr(ctx)?.try_as_length()?;

                let v = from_pos
                    .pos
                    .vec_to(to_pos.pos)
                    .try_normalize()
                    .map(|v| v.scale(dist))
                    .unwrap_or_default();

                // WARN:
                // if the two points are closer than geom::EPS together, v will be the zero vec
                // and the point "on the line" will end up at "from"

                dst.pos = from_pos.pos + v;
            }
            PointObj::OnCurve { curve, dist } => {
                let curve = ctx
                    .get_cached_as::<CurveObj>(*curve)
                    .ok_or(EvalError::UnknownDependency)?;
                let dist = dist.inner().eval_expr(ctx)?.try_as_length()?;
                dst.pos = curve.curve.point_on(dist).1;
            }
        }
        Ok(())
    }
}

impl Case<Object> for PointObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Point(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Point(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for PointVal {
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Point(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Point(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<ObjectId> for PointId {
    fn project(s: &ObjectId) -> Option<&Self> {
        match s {
            ObjectId::Point(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut ObjectId) -> Option<&mut Self> {
        match s {
            ObjectId::Point(inner) => Some(inner),
            _ => None,
        }
    }
}
