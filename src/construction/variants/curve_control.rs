use crate::construction::{EvalError, ObjectId, PointId, Variant};
use crate::geom::{Point2, Polar};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveControlId(pub(crate) usize);

impl From<usize> for CurveControlId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CurveControl {
    pub parent: PointId,
    pub off: Polar,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CurveControlVal {
    pub pos: Point2,
    pub parent: Point2,
}

impl Variant for CurveControl {
    type Id = CurveControlId;
    type Value = CurveControlVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::CurveControl(id, self, Self::Value::default())
    }

    fn eval(
        &self,
        dst: &mut Self::Value,
        ctx: &crate::construction::EvalCtx,
    ) -> Result<(), EvalError> {
        let parent = ctx.get_point_position(self.parent)?;
        dst.parent = parent;
        dst.pos = parent + self.off;

        Ok(())
    }
    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([self.parent.into()]);
    }
}
