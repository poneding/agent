use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("memory record is invalid: {0}")]
    InvalidRecord(String),
    #[error("memory embedding is invalid: {0}")]
    InvalidEmbedding(String),
    #[error("memory query is invalid: {0}")]
    InvalidQuery(String),
    #[error("memory rerank request is invalid: {0}")]
    InvalidRerank(String),
    #[error("memory proposal is invalid: {0}")]
    InvalidProposal(String),
    #[error("duplicate memory record: {0}")]
    DuplicateRecord(String),
}
