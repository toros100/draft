use crate::construction::{EvalError, ObjectId, PointId, Variant};
use crate::geom::Point2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LineId(pub(crate) usize);

impl From<usize> for LineId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub from: PointId,
    pub to: PointId,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LineVal {
    pub from: Point2,
    pub to: Point2,
}

impl Variant for Line {
    type Id = LineId;
    type Value = LineVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::Line(id, self, Self::Value::default())
    }

    fn eval(
        &self,
        dst: &mut Self::Value,
        ctx: &crate::construction::EvalCtx,
    ) -> Result<(), EvalError> {
        let from = ctx.get_point_position(self.from)?;
        let to = ctx.get_point_position(self.to)?;

        dst.from = from;
        dst.to = to;
        Ok(())
    }

    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([self.from.into(), self.to.into()]);
    }
}
