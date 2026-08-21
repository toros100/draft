use super::*;
use crate::{Case, VariantOld};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct VariableId(pub(crate) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LengthVariableId(VariableId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AngleVariableId(VariableId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ScalarVariableId(VariableId);

impl From<usize> for VariableId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl LengthVariableId {
    pub fn inner(self) -> VariableId {
        self.0
    }
}
impl AngleVariableId {
    pub fn inner(self) -> VariableId {
        self.0
    }
}
impl ScalarVariableId {
    pub fn inner(self) -> VariableId {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum VariableObj {
    Length(LengthExpression),
    Scalar(ScalarExpression),
    Angle(AngleExpression),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VariableVal {
    pub val: ExpressionVal,
}

impl Case<Object> for VariableObj {
    fn project(s: &Object) -> Option<&Self> {
        match s {
            Object::Variable(inner) => Some(inner),
            _ => None,
        }
    }
    fn project_mut(s: &mut Object) -> Option<&mut Self> {
        match s {
            Object::Variable(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Case<Value> for VariableVal {
    fn project_mut(s: &mut Value) -> Option<&mut Self> {
        match s {
            Value::Variable(inner) => Some(inner),
            _ => None,
        }
    }
    fn project(s: &Value) -> Option<&Self> {
        match s {
            Value::Variable(inner) => Some(inner),
            _ => None,
        }
    }
}

impl VariantOld<Object> for VariableObj {
    type Id = VariableId;
    type Val = VariableVal;
    type EvalError = EvalError;
    fn eval_old(
        &self,
        dst: &mut Self::Val,
        ctx: &impl crate::EvalCtxOld<Object>,
    ) -> Result<(), Self::EvalError> {
        match self {
            VariableObj::Angle(inner) => match inner.inner().eval_expr_old(ctx)? {
                val @ ExpressionVal::Angle(_) => *dst = VariableVal { val },
                _ => return Err(EvalError::UnexpectedType),
            },
            VariableObj::Length(inner) => match inner.inner().eval_expr_old(ctx)? {
                val @ ExpressionVal::Length(_) => *dst = VariableVal { val },
                _ => return Err(EvalError::UnexpectedType),
            },
            VariableObj::Scalar(inner) => match inner.inner().eval_expr_old(ctx)? {
                val @ ExpressionVal::Scalar(_) => *dst = VariableVal { val },
                _ => return Err(EvalError::UnexpectedType),
            },
        }
        Ok(())
    }
    fn dependencies_old(&self, dst: &mut impl Extend<<Object as crate::SumObject>::Id>) {
        match self {
            VariableObj::Angle(exp) => {
                exp.inner().dependencies(dst);
            }
            VariableObj::Length(exp) => {
                exp.inner().dependencies(dst);
            }
            VariableObj::Scalar(exp) => {
                exp.inner().dependencies(dst);
            }
        }
    }
}
