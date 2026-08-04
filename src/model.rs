use crate::graph::Value;
use crate::slint_generatedMainWindow;
use slint::{FilterModel, MapModel, Model, ModelNotify};
use std::cell::RefMut;
use std::rc::Rc;

use crate::graph::{Object, ObjectArena, ObjectId};

use std::cell::RefCell;

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
    obj: Object,
    val: Value,
}

impl Model for ObjectModel {
    type Data = Row;
    fn row_count(&self) -> usize {
        self.arena.borrow().v.len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let b = self.arena.borrow();
        if row >= b.v.len() {
            None
        } else {
            let id = b.ids[row];
            let obj = b.v[row].clone();

            // WARN: ugly, should obviously not just unwrap here
            // at this point (for display), everything should be evaluated
            // need to deal with the case where something is broken somehow
            let val = b.cache[row].clone().unwrap();
            Some(Row { id, obj, val })
        }
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }
}

// i wish this was stable
// type PointsModel = impl slint::Model<Data = slint_generatedMainWindow::PointData>;

pub fn points_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::PointData> {
    let points_model = FilterModel::new(om.clone(), |e| {
        matches!(
            e,
            Row {
                obj: Object::Point(_),
                ..
            }
        )
    });

    MapModel::new(points_model, {
        move |row| {
            let calculated_pos =
                    // if let Evaluated::Point(p) = cl.arena.borrow().cache[idx].unwrap() {
                    if let Value::Point(p) = row.val {
                        p
                    } else {
                        panic!("type error (unexpected Evaluated variant)")
                    };

            slint_generatedMainWindow::PointData {
                id: row.id.into(),
                pos: slint_generatedMainWindow::WorldPos {
                    x: calculated_pos.x as f32,
                    y: calculated_pos.y as f32,
                },
            }
        }
    })
}

pub fn lines_model(
    om: Rc<ObjectModel>,
) -> impl slint::Model<Data = slint_generatedMainWindow::LineData> {
    let lines_model = FilterModel::new(om.clone(), |r| {
        matches!(
            r,
            Row {
                obj: Object::Line(_),
                ..
            }
        )
    });

    MapModel::new(lines_model, |r| {
        let val = if let Value::Line(l) = r.val {
            l
        } else {
            panic!("oops")
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
    let curves_model = FilterModel::new(om.clone(), |r| {
        matches!(
            r,
            Row {
                obj: Object::Curve(_),
                ..
            }
        )
    });

    MapModel::new(curves_model, |r| {
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
    let controls_model = FilterModel::new(om.clone(), |r| {
        matches!(
            r,
            Row {
                obj: Object::CurveControl(_),
                ..
            }
        )
    });

    MapModel::new(controls_model, |r| {
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
