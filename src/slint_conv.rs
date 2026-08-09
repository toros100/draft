use crate::construction::{CurveControlObj, CurveObj, LineObj, Object, ObjectId, PointObj};
use crate::core::*;
use crate::geom::Point2;
use crate::{CurveControlData, CurveData, LineData, ObjId, PointData};

pub trait SlintData<V: Variant<S>, S: SumObject>: 'static + Clone {
    fn from_id_and_value(id: <V as Variant<S>>::Id, value: &<V as Variant<S>>::Val) -> Self;
}

pub const ID_NONE: crate::slint_generatedMainWindow::OptionObjId =
    crate::slint_generatedMainWindow::OptionObjId { raw: -1 };

impl SlintData<PointObj, Object> for PointData {
    fn from_id_and_value(
        id: <PointObj as Variant<Object>>::Id,
        value: &<PointObj as Variant<Object>>::Val,
    ) -> Self {
        PointData {
            id: <ObjId as From<ObjectId>>::from(id.into()),
            pos: value.pos.into(),
        }
    }
}

impl SlintData<LineObj, Object> for LineData {
    fn from_id_and_value(
        id: <LineObj as Variant<Object>>::Id,
        value: &<LineObj as Variant<Object>>::Val,
    ) -> Self {
        Self {
            id: <ObjId as From<ObjectId>>::from(id.into()),
            from: value.from.into(),
            to: value.to.into(),
        }
    }
}

impl SlintData<CurveControlObj, Object> for CurveControlData {
    fn from_id_and_value(
        id: <CurveControlObj as Variant<Object>>::Id,
        value: &<CurveControlObj as Variant<Object>>::Val,
    ) -> Self {
        Self {
            id: <ObjId as From<ObjectId>>::from(id.into()),
            parent: value.parent.into(),
            pos: value.pos.into(),
        }
    }
}

impl SlintData<CurveObj, Object> for CurveData {
    fn from_id_and_value(
        id: <CurveObj as Variant<Object>>::Id,
        value: &<CurveObj as Variant<Object>>::Val,
    ) -> Self {
        Self {
            id: <ObjId as From<ObjectId>>::from(id.into()),
            from: value.curve.p_0.into(),
            from_control: value.curve.p_1.into(),
            to_control: value.curve.p_2.into(),
            to: value.curve.p_3.into(),
        }
    }
}

impl From<Option<ObjectId>> for crate::slint_generatedMainWindow::OptionObjId {
    fn from(value: Option<ObjectId>) -> Self {
        match value {
            Some(i) => {
                let raw = <ObjectId as Into<usize>>::into(i) as i32;
                debug_assert!(raw >= 0);
                crate::slint_generatedMainWindow::OptionObjId { raw }
            }
            None => ID_NONE,
        }
    }
}

impl From<ObjectId> for crate::slint_generatedMainWindow::ObjId {
    fn from(value: ObjectId) -> Self {
        let raw = <ObjectId as Into<usize>>::into(value) as i32;
        debug_assert!(raw >= 0);
        Self { raw }
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
