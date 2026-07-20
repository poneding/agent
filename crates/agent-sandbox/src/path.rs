//! Workspace path confinement.
//!
//! This is the real sandbox used by builtin tools today:
//! - [`WorkspaceSandbox::resolve_read_path`] canonicalizes and rejects escapes
//! - [`WorkspaceSandbox::resolve_write_path`] normalizes uncreated paths and
//!   validates that the nearest existing parent stays under the workspace root
//!
//! Shell tools combine this with env allowlisting + timeouts. OS-level process
//! isolation (seatbelt/bubblewrap/landlock) is intentionally out of scope for
//! this crate until a dedicated process sandbox lands; do not treat
//! `PlatformCapabilityReport` labels as enforcement.

use std::path::{Component, Path, PathBuf};

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

    pub fn resolve_write_path(&self, path: &str) -> Result<PathBuf, SandboxError> {
        if path.trim().is_empty() {
            return Err(SandboxError::EmptyPath);
        }

        let input = Path::new(path);
        let candidate = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.root.join(input)
        };
        let resolved = normalize_path(&candidate);
        if !resolved.starts_with(&self.root) {
            return Err(SandboxError::PathEscape(path.to_string()));
        }

        let existing_parent = nearest_existing_parent(&resolved).ok_or_else(|| {
            SandboxError::InvalidRoot(format!("{} has no existing parent", resolved.display()))
        })?;
        let canonical_parent = existing_parent
            .canonicalize()
            .map_err(|error| SandboxError::InvalidRoot(error.to_string()))?;
        if !canonical_parent.starts_with(&self.root) {
            return Err(SandboxError::PathEscape(path.to_string()));
        }

        Ok(resolved)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn nearest_existing_parent(path: &Path) -> Option<&Path> {
    path.ancestors().skip(1).find(|ancestor| ancestor.exists())
}
