use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("memory record is invalid: {0}")]
    InvalidRecord(String),
    #[error("memory embedding is invalid: {0}")]
    InvalidEmbedding(String),
    #[error("duplicate memory record: {0}")]
    DuplicateRecord(String),
}
