use crate::construction::{CurveId, EvalCtx, EvalError, ObjectId, Variant};
use crate::expression::{AngleExpression, LengthExpression};
use crate::geom::{CubicBezier, Point2};
use std::iter::Extend;

#[derive(Debug, Default, Clone, Copy)]
pub struct PointFreeVal {
    pub pos: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, derive_more::From, Hash, Eq)]
pub enum PointId {
    Free(PointFreeId),
    DistAngle(PointDistAngleId),
    OnLine(PointOnLineId),
    OnCurve(PointOnCurveId),
}

impl From<PointId> for ObjectId {
    fn from(value: PointId) -> Self {
        match value {
            PointId::Free(inner) => inner.into(),
            PointId::DistAngle(inner) => inner.into(),
            PointId::OnLine(inner) => inner.into(),
            PointId::OnCurve(inner) => inner.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointFreeId(pub(crate) usize);

#[derive(Debug, Clone)]
pub struct PointFree {
    pub pos: Point2,
}

impl From<usize> for PointFreeId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Variant for PointFree {
    type Id = PointFreeId;
    type Value = PointFreeVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::PointFree(id, self, Self::Value::default())
    }

    fn eval(&self, dst: &mut Self::Value, _: &EvalCtx) -> Result<(), EvalError> {
        dst.pos = self.pos;
        Ok(())
    }

    fn dependencies(&self, _: &mut impl Extend<ObjectId>) {}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointDistAngleId(pub(crate) usize);

impl From<usize> for PointDistAngleId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct PointDistAngle {
    pub parent: PointId,
    pub dist: LengthExpression,
    pub lock_dist: bool,
    pub angle: AngleExpression,
    pub lock_angle: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PointDistAngleVal {
    pub parent: Point2,
    pub pos: Point2,
}

impl Variant for PointDistAngle {
    type Id = PointDistAngleId;
    type Value = PointDistAngleVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::PointDistAngle(id, self, Self::Value::default())
    }

    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([self.parent.into()]);
        self.dist.inner().dependencies(dst);
        self.angle.inner().dependencies(dst);
    }

    fn eval(&self, dst: &mut Self::Value, ctx: &EvalCtx) -> Result<(), EvalError> {
        let parent = ctx.get_point_position(self.parent)?;
        let dist = self.dist.inner().eval_expr(ctx)?.try_as_length()?;
        let angle = self.angle.inner().eval_expr(ctx)?.try_as_angle()?;
        dst.parent = parent;
        dst.pos = parent + crate::geom::polar(dist.into(), angle.into());

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointOnLineId(pub(crate) usize);

impl From<usize> for PointOnLineId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct PointOnLine {
    pub from: PointId,
    pub to: PointId,
    pub dist: LengthExpression,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PointOnLineVal {
    pub from: Point2,
    pub to: Point2,
    pub pos: Point2,
}

impl Variant for PointOnLine {
    type Id = PointOnLineId;
    type Value = PointOnLineVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::PointOnLine(id, self, Self::Value::default())
    }

    fn eval(&self, dst: &mut Self::Value, ctx: &EvalCtx) -> Result<(), EvalError> {
        let from = ctx.get_point_position(self.from)?;
        let to = ctx.get_point_position(self.to)?;
        let dist = self.dist.inner().eval_expr(ctx)?.try_as_length()?;
        let dir = (to - from).try_normalize().unwrap_or_default();
        dst.pos = from + (f64::from(dist) * dir);
        dst.from = from;
        dst.to = to;
        Ok(())
    }
    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([self.from.into(), self.to.into()]);
        self.dist.inner().dependencies(dst);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PointOnCurveId(pub(crate) usize);

impl From<usize> for PointOnCurveId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct PointOnCurve {
    pub curve: CurveId,
    pub dist: LengthExpression,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PointOnCurveVal {
    // duplicate information but avoids having to look it up? not sure if
    // worth it
    pub curve: CubicBezier,
    pub pos: Point2,
    pub t: f64,
}

impl Variant for PointOnCurve {
    type Id = PointOnCurveId;
    type Value = PointOnCurveVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::PointOnCurve(id, self, Self::Value::default())
    }

    fn eval(&self, dst: &mut Self::Value, ctx: &EvalCtx) -> Result<(), EvalError> {
        let curve = ctx.get_curve(self.curve)?.curve;
        let dist = self.dist.inner().eval_expr(ctx)?.try_as_length()?;

        let (t, p) = curve.point_on(dist.into());
        dst.t = t;
        dst.pos = p;
        dst.curve = curve;
        Ok(())
    }

    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([self.curve.into()]);
        self.dist.inner().dependencies(dst);
    }
}
