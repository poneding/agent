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

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PluginProcessError {
    #[error("invalid plugin process config: {0}")]
    InvalidConfig(String),
    #[error("failed to spawn plugin process: {0}")]
    Spawn(String),
    #[error("plugin process exited unsuccessfully with code {code:?}: {stderr}")]
    Exit { code: Option<i32>, stderr: String },
    #[error("plugin process timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("plugin process stdout exceeded {limit} bytes with {actual} bytes")]
    OutputTooLarge { limit: usize, actual: usize },
    #[error("plugin process protocol error: {0}")]
    Protocol(String),
    #[error("plugin process returned error {error_code}: {message}")]
    Plugin {
        error_code: String,
        message: String,
        recoverable: bool,
    },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PluginWasmError {
    #[error("invalid WASM config: {0}")]
    InvalidConfig(String),
    #[error("WASM module exceeded {limit} bytes with {actual} bytes")]
    ModuleTooLarge { limit: usize, actual: usize },
    #[error("invalid WASM module: {0}")]
    InvalidModule(String),
    #[error("unsupported WASM host call: {name}")]
    UnsupportedHostCall { name: String },
}
