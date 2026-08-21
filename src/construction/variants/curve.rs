use crate::construction::{CurveControlId, EvalError, ObjectId, PointId, Variant};
use crate::geom;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CurveId(pub(crate) usize);

impl From<usize> for CurveId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Curve {
    pub from: PointId,
    pub to: PointId,
    pub from_control: CurveControlId,
    pub to_control: CurveControlId,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CurveVal {
    pub curve: geom::CubicBezier,
}

impl Variant for Curve {
    type Id = CurveId;
    type Value = CurveVal;

    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::Curve(id, self, Self::Value::default())
    }

    fn dependencies(&self, dst: &mut impl Extend<ObjectId>) {
        dst.extend([
            self.from.into(),
            self.to.into(),
            self.from_control.into(),
            self.to_control.into(),
        ]);
    }

    fn eval(
        &self,
        dst: &mut Self::Value,
        ctx: &crate::construction::EvalCtx,
    ) -> Result<(), EvalError> {
        let from = ctx.get_point_position(self.from)?;
        let to = ctx.get_point_position(self.to)?;
        let from_control = ctx.get_curve_control(self.from_control)?;
        let to_control = ctx.get_curve_control(self.to_control)?;
        dst.curve = geom::curve(from, from_control, to_control, to);
        Ok(())
    }
}
