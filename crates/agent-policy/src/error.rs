use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PolicyError {
    #[error("invalid permission request: {0}")]
    InvalidRequest(String),
}
