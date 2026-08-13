use std::fmt::Debug;

pub trait Case<T>: Into<T> {
    fn project(s: &T) -> Option<&Self>;
    fn project_mut(s: &mut T) -> Option<&mut Self>;
}

pub trait CaseExt: Sized {
    fn case<V: Case<Self>>(&self) -> Option<&V> {
        V::project(self)
    }
    fn case_mut<V: Case<Self>>(&mut self) -> Option<&mut V> {
        V::project_mut(self)
    }
}

impl<T> CaseExt for T {}

pub trait SumObject: Sized {
    type Id: Copy;
    type Value;
    type EvalError: Debug;
    fn eval_dispatch(
        &self,
        dst: &mut Option<Self::Value>,
        ctx: &impl EvalCtx<Self>,
    ) -> Result<(), Self::EvalError>;
}

pub trait EvalCtx<S: SumObject> {
    // TODO: would be better to return error
    fn get_cached(&self, id: S::Id) -> Option<&S::Value>;
    fn get_obj(&self, id: S::Id) -> Option<&S>;
}

pub trait EvalCtxExt<S: SumObject>: EvalCtx<S> {
    fn get_cached_as<'a, V>(&'a self, id: V::Id) -> Option<&'a V::Val>
    where
        V: Variant<S>,
        S::Value: 'a,
    {
        self.get_cached(id.into())?.case()
    }

    #[allow(unused)]
    fn get_obj_as<'a, V>(&'a self, id: V::Id) -> Option<&'a V>
    where
        V: Variant<S>,
        S: 'a,
    {
        self.get_obj(id.into())?.case()
    }
}

impl<S, T> EvalCtxExt<S> for T
where
    S: SumObject,
    T: EvalCtx<S>,
{
}

pub trait Variant<S: SumObject>: Case<S> {
    type Id: Case<S::Id> + From<usize> + Copy;
    type Val: Case<S::Value>;
    type EvalError: Debug;
    fn dependencies(&self, dst: &mut impl Extend<S::Id>);
    fn eval(&self, dst: &mut Self::Val, ctx: &impl EvalCtx<S>) -> Result<(), Self::EvalError>;
}
