use crate::construction::value::{TryProject, Value};
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
