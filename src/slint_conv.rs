use crate::construction::{CurveControlId, CurveId, LineId, ObjectId, PointId};
use crate::geom::Point2;
use crate::{TaggedObjectId, core::*};

pub trait SlintData<V: Variant<S>, S: SumObject>: 'static + Clone {
    fn from_id_and_value(id: <V as Variant<S>>::Id, value: &<V as Variant<S>>::Val) -> Self;
}

impl From<TaggedObjectId> for ObjectId {
    fn from(value: TaggedObjectId) -> Self {
        match value.kind {
            crate::ObjectKind::PointDistAngle
            | crate::ObjectKind::PointFree
            | crate::ObjectKind::PointOnLine
            | crate::ObjectKind::PointOnCurve => PointId::from(value.raw as usize).into(),
            crate::ObjectKind::Line => LineId::from(value.raw as usize).into(),
            crate::ObjectKind::Curve => CurveId::from(value.raw as usize).into(),
            crate::ObjectKind::CurveControl => CurveControlId::from(value.raw as usize).into(),
        }
    }
}

impl From<crate::slint_generatedMainWindow::WorldPos> for Point2 {
    fn from(value: crate::slint_generatedMainWindow::WorldPos) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
        }
    }
}

impl From<Point2> for crate::slint_generatedMainWindow::WorldPos {
    fn from(value: Point2) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}
