use std::collections::HashMap;
use std::hash::Hash;

use crate::core::*;

struct PrefixCtx<'a, S: SumObject> {
    id_to_idx: &'a HashMap<S::Id, usize>,
    vals: &'a [Option<S::Value>],
}

impl<'a, S: SumObject> EvalCtx<S> for PrefixCtx<'a, S>
where
    S::Id: PartialEq + Eq + Hash,
{
    fn get_cached(&self, id: <S as SumObject>::Id) -> Option<&<S as SumObject>::Value> {
        let idx = *self.id_to_idx.get(&id)?;
        self.vals[idx].as_ref()
    }
}

pub struct Arena<S: SumObject> {
    pub(crate) objs: Vec<(S::Id, S)>,
    pub(crate) ids: Vec<S::Id>,
    pub(crate) vals: Vec<Option<S::Value>>,
    pub(crate) id_to_idx: HashMap<S::Id, usize>,
    next_id_raw: usize,
    dep_scratch: Vec<S::Id>,
    #[allow(unused)] // TODO: track somehow?
    min_dirty: Option<usize>,
}

impl<S: SumObject> Default for Arena<S> {
    fn default() -> Self {
        // derived Default has unwanted bounds
        Self {
            objs: Default::default(),
            ids: Default::default(),
            id_to_idx: Default::default(),
            next_id_raw: Default::default(),
            dep_scratch: Default::default(),
            min_dirty: Default::default(),
            vals: Default::default(),
        }
    }
}

impl<S> Arena<S>
where
    S: SumObject,
    S::Id: PartialEq + Eq + Hash,
{
    pub fn try_push_obj<V: Variant<S>>(&mut self, o: V) -> V::Id {
        self.dep_scratch.clear();
        o.dependencies(&mut self.dep_scratch);

        for dep_id in self.dep_scratch.iter() {
            if !self.id_to_idx.contains_key(dep_id) {
                panic!("missing dep")
            }
        }

        let variant_id = V::Id::from(self.next_id_raw);
        let id = variant_id.into();
        self.next_id_raw += 1;
        let idx = self.objs.len();

        self.id_to_idx.insert(id, idx);
        self.objs.push((id, o.into()));
        self.ids.push(id);

        // at this point we know the variants value type, could push it
        // but that would require a C::Val: Default bound here

        self.vals.push(None);
        variant_id
    }

    pub fn evaluate_all(&mut self) {
        for i in 0..self.objs.len() {
            let (prev, rest) = self.vals.split_at_mut(i);

            // having a concrete implementation of EvalCtx baked into this generic impl is a bit ugly

            let ctx = PrefixCtx {
                vals: prev,
                id_to_idx: &self.id_to_idx,
            };

            let v = &mut rest[0];
            self.objs[i].1.eval_dispatch(v, &ctx).unwrap();
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
                self.ids[idx],
                &self.objs[idx].1,
                self.vals[idx].as_ref().expect("hacky"),
            ))
        }
    }
}
