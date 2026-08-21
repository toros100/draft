use crate::{
    construction::Variant,
    expression::{Length, LengthExpression},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, derive_more::From)]
pub struct LengthVariableId(pub(crate) usize);

#[derive(Debug, Clone)]
pub struct LengthVariable {
    pub expr: LengthExpression,
}

impl Variant for LengthVariable {
    type Id = LengthVariableId;
    type Value = Length;
    fn eval(
        &self,
        dst: &mut Self::Value,
        ctx: &crate::construction::EvalCtx,
    ) -> Result<(), crate::construction::EvalError> {
        *dst = self.expr.inner().eval_expr(ctx)?.try_as_length()?;
        Ok(())
    }
    fn into_entry(self, id: Self::Id) -> crate::construction::Entry {
        crate::construction::Entry::LengthVariable(id, self, Length::default())
    }

    fn dependencies(&self, dst: &mut impl Extend<crate::construction::ObjectId>) {
        self.expr.inner().dependencies(dst);
    }
}
