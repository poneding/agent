use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AgentTypesError {
    #[error("event sequence must be non-zero: {0}")]
    InvalidEventSequence(u64),
    #[error("schema version must be positive: {0}")]
    InvalidSchemaVersion(u16),
    #[error("timestamp must be UTC")]
    NonUtcTimestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl AgentError {
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}
