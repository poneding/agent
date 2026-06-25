use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DesktopError {
    #[error("invalid desktop session: {0}")]
    InvalidSession(String),
}
