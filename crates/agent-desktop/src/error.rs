use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DesktopError {
    #[error("invalid desktop session: {0}")]
    InvalidSession(String),
    #[error("invalid desktop snapshot: {0}")]
    InvalidSnapshot(String),
    #[error("invalid desktop permission dialog: {0}")]
    InvalidPermission(String),
    #[error("invalid desktop diff review: {0}")]
    InvalidDiff(String),
    #[error("invalid desktop trace timeline: {0}")]
    InvalidTrace(String),
}
