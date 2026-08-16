use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::core::*;

struct PrefixCtx<'a, S: SumObject> {
    id_to_idx: &'a HashMap<S::Id, usize>,
    objs: &'a [(S::Id, S)],
    vals: &'a [Option<S::Value>],
}

impl<'a, S> EvalCtx<S> for PrefixCtx<'a, S>
where
    S: SumObject,
    S::Id: PartialEq + Eq + Hash,
{
    fn get_cached(&self, id: <S as SumObject>::Id) -> Option<&<S as SumObject>::Value> {
        let idx = *self.id_to_idx.get(&id)?;
        self.vals[idx].as_ref()
    }
    fn get_obj(&self, id: <S as SumObject>::Id) -> Option<&S> {
        let idx = *self.id_to_idx.get(&id)?;
        Some(&self.objs[idx].1)
    }
}

pub struct Arena<S: SumObject> {
    pub(crate) objs: Vec<(S::Id, S)>,
    pub(crate) vals: Vec<Option<S::Value>>,
    pub(crate) id_to_idx: HashMap<S::Id, usize>,
    pub(crate) depependents: HashMap<S::Id, HashSet<S::Id>>,
    next_id_raw: usize,
    pub(crate) dep_scratch: Vec<S::Id>,
    #[allow(unused)] // TODO: track somehow?
    min_dirty: Option<usize>,
}

impl<S: SumObject> Default for Arena<S> {
    fn default() -> Self {
        // derived Default has unwanted bounds
        Self {
            objs: Default::default(),
            // ids: Default::default(),
            id_to_idx: Default::default(),
            next_id_raw: Default::default(),
            dep_scratch: Default::default(),
            min_dirty: Default::default(),
            vals: Default::default(),
            depependents: Default::default(),
        }
    }
}

impl<S> Arena<S>
where
    S: SumObject,
    S::Id: PartialEq + Eq + Hash,
{
    pub fn next_id<V>(&mut self) -> V::Id
    where
        V: Variant<S>,
    {
        let id = V::Id::from(self.next_id_raw);
        self.next_id_raw += 1;
        id
    }

    pub fn try_push_obj<V: Variant<S>>(&mut self, o: V) -> V::Id {
        self.dep_scratch.clear();
        o.dependencies(&mut self.dep_scratch);

        let variant_id = V::Id::from(self.next_id_raw);
        let id = variant_id.into();
        self.next_id_raw += 1;

        for dep_id in self.dep_scratch.iter() {
            if !self.id_to_idx.contains_key(dep_id) {
                panic!("missing dep")
            } else {
                let dep = self.depependents.get_mut(dep_id).unwrap();
                dep.insert(id);
            }
        }

        // let variant_id =
        // let id = variant_id.into();
        // self.next_id_raw += 1;
        let idx = self.objs.len();

        self.id_to_idx.insert(id, idx);

        self.depependents.insert(id, HashSet::new());

        self.objs.push((id, o.into()));
        // self.ids.push(id);

        // at this point we know the variants value type, could push it
        // but that would require a C::Val: Default bound here

        self.vals.push(None);
        variant_id
    }

    pub fn evaluate_all(&mut self) {
        for i in 0..self.objs.len() {
            let (val_pref, val_rest) = self.vals.split_at_mut(i);

            let (obj_pref, obj_rest) = self.objs.split_at_mut(i);

            // having a concrete implementation of EvalCtx baked into this generic impl is a bit ugly

            let ctx = PrefixCtx {
                vals: val_pref,
                id_to_idx: &self.id_to_idx,
                objs: obj_pref,
            };

            let v = &mut val_rest[0];
            let o = &obj_rest[0].1;
            o.eval_dispatch(v, &ctx).unwrap();
        }
    }

    pub fn len(&self) -> usize {
        self.objs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_by_index(&self, idx: usize) -> Option<(S::Id, &S, &S::Value)> {
        if idx >= self.len() {
            None
        } else {
            Some((
                self.objs[idx].0,
                &self.objs[idx].1,
                self.vals[idx].as_ref().expect("hacky"),
            ))
        }
    }
}
