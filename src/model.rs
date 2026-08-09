use crate::arena::Arena;
use crate::core::*;
use crate::slint_conv::SlintData;
use slint::{FilterModel, MapModel, Model, ModelNotify, ModelRc};
use std::cell::{RefCell, RefMut};
use std::hash::Hash;
use std::rc::Rc;

pub struct ArenaModel<S: SumObject> {
    arena: RefCell<Arena<S>>,
    notify: ModelNotify,
}

impl<S: SumObject> Default for ArenaModel<S> {
    fn default() -> Self {
        Self {
            arena: RefCell::new(Arena::default()),
            notify: ModelNotify::default(),
        }
    }
}

impl<S: SumObject> ArenaModel<S> {
    pub fn arena_mut(&self) -> RefMut<'_, Arena<S>> {
        self.arena.borrow_mut()
    }

    pub fn notify_all(&self) {
        self.notify.reset();
    }
}

pub struct Row<S: SumObject> {
    id: S::Id,
    val: S::Value,
}

impl<S: SumObject> Model for ArenaModel<S>
where
    S::Id: PartialEq + Eq + Hash,
    S::Value: Clone,
{
    type Data = Row<S>;
    fn row_count(&self) -> usize {
        self.arena.borrow().len()
    }
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let ar = self.arena.borrow();
        if row >= ar.len() {
            None
        } else {
            let (a, _, c) = ar.get_by_index(row)?;
            Some(Row {
                id: a,
                val: c.clone(),
            })
        }
    }

    fn model_tracker(&self) -> &dyn slint::ModelTracker {
        &self.notify
    }
}

pub fn filter_map_model<S, V, D>(am: Rc<ArenaModel<S>>) -> ModelRc<D>
where
    S: SumObject + 'static,
    V: Variant<S>,
    D: SlintData<V, S>,
    S::Id: PartialEq + Eq + Hash,
    S::Value: Clone,
{
    let filtered = FilterModel::new(am.clone(), |r| V::Id::project(&r.id).is_some());

    ModelRc::new(MapModel::new(filtered, |r| {
        let v = r.val.case().unwrap();
        let id = *r.id.case().unwrap();
        D::from_id_and_value(id, v)
    }))
}
