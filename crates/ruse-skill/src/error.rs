use std::path::PathBuf;

use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SkillError {
    #[error("skill root not found: {0}")]
    RootNotFound(PathBuf),
    #[error("skill manifest not found: {0}")]
    ManifestNotFound(PathBuf),
    #[error("invalid skill manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid skill prompt: {0}")]
    InvalidPrompt(String),
    #[error("failed to read skill file {path}: {message}")]
    Io { path: PathBuf, message: String },
}
