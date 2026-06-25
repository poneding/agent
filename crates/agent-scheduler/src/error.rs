use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SchedulerError {
    #[error("invalid task: {0}")]
    InvalidTask(String),
    #[error("invalid task transition: {0}")]
    InvalidTransition(String),
    #[error("invalid task lease: {0}")]
    InvalidLease(String),
}
