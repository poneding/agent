use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SandboxError {
    #[error("workspace root is invalid: {0}")]
    InvalidRoot(String),
    #[error("path cannot be empty")]
    EmptyPath,
    #[error("path does not exist: {0}")]
    NotFound(String),
    #[error("path escapes workspace: {0}")]
    PathEscape(String),
}

#[derive(Clone, Debug)]
pub struct WorkspaceSandbox {
    root: PathBuf,
}

impl WorkspaceSandbox {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, SandboxError> {
        let root = root
            .as_ref()
            .canonicalize()
            .map_err(|error| SandboxError::InvalidRoot(error.to_string()))?;

        if !root.is_dir() {
            return Err(SandboxError::InvalidRoot(format!(
                "{} is not a directory",
                root.display()
            )));
        }

        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve_read_path(&self, path: &str) -> Result<PathBuf, SandboxError> {
        if path.trim().is_empty() {
            return Err(SandboxError::EmptyPath);
        }

        let input = Path::new(path);
        let candidate = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.root.join(input)
        };
        let resolved = candidate
            .canonicalize()
            .map_err(|_| SandboxError::NotFound(path.to_string()))?;

        if !resolved.starts_with(&self.root) {
            return Err(SandboxError::PathEscape(path.to_string()));
        }

        Ok(resolved)
    }
}
