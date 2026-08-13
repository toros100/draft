use crate::construction::ObjectId;
use crate::core::*;
use crate::geom::Point2;

pub trait SlintData<V: Variant<S>, S: SumObject>: 'static + Clone {
    fn from_id_and_value(id: <V as Variant<S>>::Id, value: &<V as Variant<S>>::Val) -> Self;
}

pub const ID_NONE: crate::slint_generatedMainWindow::OptionObjId =
    crate::slint_generatedMainWindow::OptionObjId { raw: -1 };

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
