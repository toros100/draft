use crate::construction::object::ArenaObject;
use crate::construction::object::CurveControlObj;
use crate::construction::object::CurveObj;
use crate::construction::object::LineObj;
use crate::construction::object::ObjectId;
use crate::construction::object::PointObj;
use crate::geom::Point2;
use crate::model::SlintData;

pub const ID_NONE: crate::slint_generatedMainWindow::OptionObjId =
    crate::slint_generatedMainWindow::OptionObjId { raw: -1 };

impl SlintData<PointObj> for crate::slint_generatedMainWindow::PointData {
    fn from_value(
        id: <PointObj as ArenaObject>::Id,
        value: &<PointObj as ArenaObject>::Val,
    ) -> Self {
        let id_erased: ObjectId = id.into();
        Self {
            id: id_erased.into(),
            pos: value.pos.into(),
        }
    }
}
impl SlintData<LineObj> for crate::slint_generatedMainWindow::LineData {
    fn from_value(id: <LineObj as ArenaObject>::Id, value: &<LineObj as ArenaObject>::Val) -> Self {
        let id_erased: ObjectId = id.into();
        Self {
            id: id_erased.into(),
            from: value.from.into(),
            to: value.to.into(),
        }
    }
}

impl SlintData<CurveControlObj> for crate::slint_generatedMainWindow::CurveControlData {
    fn from_value(
        id: <CurveControlObj as ArenaObject>::Id,
        value: &<CurveControlObj as ArenaObject>::Val,
    ) -> Self {
        let id_erased: ObjectId = id.into();
        Self {
            id: id_erased.into(),
            parent: value.parent.into(),
            pos: value.pos.into(),
        }
    }
}

impl SlintData<CurveObj> for crate::slint_generatedMainWindow::CurveData {
    fn from_value(
        id: <CurveObj as ArenaObject>::Id,
        value: &<CurveObj as ArenaObject>::Val,
    ) -> Self {
        let id_erased: ObjectId = id.into();
        Self {
            id: id_erased.into(),
            from: value.from.into(),
            from_control: value.control_1.into(),
            to_control: value.control_2.into(),
            to: value.to.into(),
        }
    }
}

impl From<Option<ObjectId>> for crate::slint_generatedMainWindow::OptionObjId {
    fn from(value: Option<ObjectId>) -> Self {
        match value {
            Some(i) => {
                let raw = i.into_raw() as i32;
                debug_assert!(raw >= 0);
                crate::slint_generatedMainWindow::OptionObjId { raw }
            }
            None => ID_NONE,
        }
    }
}

impl From<ObjectId> for crate::slint_generatedMainWindow::ObjId {
    fn from(value: ObjectId) -> Self {
        let raw = value.into_raw() as i32;
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
