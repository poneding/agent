use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ContextError {
    #[error("prompt layer is invalid: {0}")]
    InvalidLayer(String),
    #[error("context budget is invalid: {0}")]
    InvalidBudget(String),
    #[error("context budget exceeded: {0}")]
    BudgetExceeded(String),
    #[error("pinned message is invalid: {0}")]
    InvalidPin(String),
    #[error("compaction request is invalid: {0}")]
    InvalidCompaction(String),
    #[error("project context is invalid: {0}")]
    InvalidProjectContext(String),
    #[error("context retrieval is invalid: {0}")]
    InvalidRetrieval(String),
}
