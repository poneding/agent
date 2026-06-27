use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("memory record is invalid: {0}")]
    InvalidRecord(String),
    #[error("memory embedding is invalid: {0}")]
    InvalidEmbedding(String),
    #[error("memory query is invalid: {0}")]
    InvalidQuery(String),
    #[error("duplicate memory record: {0}")]
    DuplicateRecord(String),
}
