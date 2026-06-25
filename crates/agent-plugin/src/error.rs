use thiserror::Error;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PluginManifestError {
    #[error("plugin manifest parse failed: {0}")]
    Parse(String),
    #[error("invalid plugin name: {0}")]
    InvalidName(String),
    #[error("unsupported plugin manifest schema version {found}; supported version is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("incompatible plugin runtime: {0}")]
    IncompatibleRuntime(String),
    #[error("invalid plugin isolation: {0}")]
    InvalidIsolation(String),
    #[error("duplicate plugin contribution: {0}")]
    DuplicateContribution(String),
    #[error("invalid plugin manifest: {0}")]
    InvalidManifest(String),
}
