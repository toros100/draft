use crate::construction::value::{TryProject, Value};
use crate::slint_generatedMainWindow;
use slint::{FilterModel, MapModel, Model, ModelNotify};
use std::cell::RefMut;
use std::rc::Rc;

use crate::construction::ObjectArena;
use crate::construction::object::{ArenaObject, ObjectId};

use std::cell::RefCell;

pub trait SlintData<A: ArenaObject>: 'static + Clone {
    fn from_value(id: A::Id, value: &A::Val) -> Self;
}

#[derive(Default)]
pub struct ObjectModel {
    arena: RefCell<ObjectArena>,
    notify: ModelNotify,
}

impl ObjectModel {
    pub fn reset(&self) {
        self.notify.reset();
    }

    pub fn arena_mut(&self) -> RefMut<'_, ObjectArena> {
        self.arena.borrow_mut()
    }
}

pub struct Row {
    id: ObjectId,
    val: Value,
}

impl Model for ObjectModel {
    type Data = Row;
    fn row_count(&self) -> usize {
        self.arena.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let b = self.arena.borrow();
        if row >= b.len() {
            None
        } else {
            let id = b.get_object(row).1;

            // WARN: ugly, should obviously not just unwrap here
            // at this point (for display), everything should be evaluated
            // need to deal with the case where something is broken somehow
            let val = b.get_value(row).unwrap().clone();
            Some(Row { id, val })
        }
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }
}

pub fn points_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::PointData> {
    let points_filter = FilterModel::new(om.clone(), |r| matches!(r.id, ObjectId::Point(_)));

    MapModel::new(points_filter, {
        move |row| {
            let calculated_pos = if let Value::Point(p) = row.val {
                p
            } else {
                panic!("type error (unexpected Evaluated variant)")
            };

            slint_generatedMainWindow::PointData {
                id: row.id.into(),
                pos: slint_generatedMainWindow::WorldPos {
                    x: calculated_pos.pos.x as f32,
                    y: calculated_pos.pos.y as f32,
                },
            }
        }
    })
}

pub fn filter_map<A, S>(om: Rc<ObjectModel>) -> impl slint::Model<Data = S>
where
    A: ArenaObject,
    S: SlintData<A>,
{
    let filtered = FilterModel::new(om.clone(), |r| A::Val::try_project(&r.val).is_ok());

    MapModel::new(filtered, |r| {
        let v = A::Val::try_project(&r.val).expect("value type should still match after filtering");
        let id = A::Id::try_from(r.id).expect("id type should match value type");
        S::from_value(id, v)
    })
}

pub fn lines_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::LineData> {
    let lines_filter = FilterModel::new(om.clone(), |r| matches!(r.id, ObjectId::Line(_)));

    MapModel::new(lines_filter, |r| {
        let val = if let Value::Line(l) = r.val {
            l
        } else {
            panic!("excluded by filter (and id kind matching value kind lol)")
        };

        slint_generatedMainWindow::LineData {
            from: val.from.into(),
            to: val.to.into(),
            id: r.id.into(),
        }
    })
}

pub fn curves_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::CurveData> {
    let curves_filter = FilterModel::new(om.clone(), |r| matches!(r.id, ObjectId::Curve(_)));

    MapModel::new(curves_filter, |r| {
        let val = if let Value::Curve(l) = r.val {
            l
        } else {
            panic!("oops")
        };

        slint_generatedMainWindow::CurveData {
            from: val.from.into(),
            to: val.to.into(),
            from_control: val.control_1.into(),
            to_control: val.control_2.into(),
            id: r.id.into(),
        }
    })
}

pub fn curve_controls_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::CurveControlData> {
    let curve_controls_filter =
        FilterModel::new(om.clone(), |r| matches!(r.id, ObjectId::CurveControl(_)));

    MapModel::new(curve_controls_filter, |r| {
        let val = if let Value::CurveControl(l) = r.val {
            l
        } else {
            panic!("oops")
        };

        slint_generatedMainWindow::CurveControlData {
            id: r.id.into(),
            parent: val.parent.into(),
            pos: val.pos.into(),
        }
    })
}
