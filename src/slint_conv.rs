use crate::construction::object::ObjectId;
use crate::geom::Point2;

pub const ID_NONE: crate::slint_generatedMainWindow::OptionObjId =
    crate::slint_generatedMainWindow::OptionObjId { raw: -1 };

impl From<crate::slint_generatedMainWindow::OptionObjId> for Option<ObjectId> {
    fn from(value: crate::slint_generatedMainWindow::OptionObjId) -> Self {
        match value.raw {
            v if v == ID_NONE.raw => None,
            v if v >= 0 => Some(ObjectId::from_raw(v as usize)),
            _ => {
                if cfg!(debug_assertions) {
                    panic!("unexpected raw id value")
                }
                None
            }
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

impl From<crate::slint_generatedMainWindow::ObjId> for ObjectId {
    fn from(value: crate::slint_generatedMainWindow::ObjId) -> Self {
        debug_assert!(value.raw >= 0);
        Self::from_raw(value.raw as usize)
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
