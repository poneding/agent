#![forbid(unsafe_code)]
//! Workspace sandbox contracts for agent.

pub mod path;
pub mod platform;

pub use path::{SandboxError, WorkspaceSandbox};
pub use platform::{
    CapabilityStatus, PlatformCapabilityReport, PlatformReportError, PluginIsolationCapability,
    SandboxCapability, SecretProviderCapability,
};
