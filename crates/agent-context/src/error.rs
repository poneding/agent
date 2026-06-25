use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ContextError {
    #[error("prompt layer is invalid: {0}")]
    InvalidLayer(String),
    #[error("context budget is invalid: {0}")]
    InvalidBudget(String),
    #[error("context budget exceeded: {0}")]
    BudgetExceeded(String),
}
