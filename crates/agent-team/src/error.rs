use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TeamError {
    #[error("invalid subagent handoff: {0}")]
    InvalidHandoff(String),
    #[error("invalid handoff transition: {0}")]
    InvalidTransition(String),
    #[error("invalid team blackboard entry: {0}")]
    InvalidBlackboard(String),
}
