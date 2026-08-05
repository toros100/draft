use crate::construction::expression::ExpressionError;
use crate::construction::object::ObjectId;
use crate::construction::value::Value;
use std::borrow::Borrow;
use thiserror::Error;

pub trait EvalCtx {
    // returning reference here because i anticipate future Value variants that are not Copy
    // (e.g. complex curve with n points)
    // using impl Borrow<ObjectId> because i often end up with &ObjectId in some destructured thing,
    // and it was annoying to always manually deref it to call the trait methods
    fn try_get(&self, id: impl Borrow<ObjectId>) -> Result<&Value, EvalError>;

    fn try_get_as<'a, T: TryFrom<&'a Value, Error = EvalError>>(
        &'a self,
        id: impl Borrow<ObjectId>,
    ) -> Result<T, EvalError> {
        self.try_get(id)?.try_into()
    }
}

pub trait Eval {
    type Output;
    fn eval(&self, ctx: &impl EvalCtx) -> Result<Self::Output, EvalError>;
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("unresolved dependency (order broken)")]
    UnresolvedDependency,
    #[error("unknown dependency")]
    UnknownDependency,
    #[error("unexpected value type")]
    UnexpectedType,
    #[error("expression error: {}", .0)]
    ExpressionError(ExpressionError),
}

impl From<ExpressionError> for EvalError {
    fn from(value: ExpressionError) -> Self {
        EvalError::ExpressionError(value)
    }
}
