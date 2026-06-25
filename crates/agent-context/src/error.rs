use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ContextError {
    #[error("prompt layer is invalid: {0}")]
    InvalidLayer(String),
}
