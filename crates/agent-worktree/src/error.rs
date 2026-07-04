use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum WorktreeError {
    #[error("invalid worktree request: {0}")]
    InvalidRequest(String),
    #[error("worktree lease not found: {0}")]
    LeaseNotFound(String),
    #[error("invalid worktree lease transition: {0}")]
    InvalidTransition(String),
}
