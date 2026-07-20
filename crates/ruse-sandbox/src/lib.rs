#![forbid(unsafe_code)]
//! Workspace sandbox contracts for agent.
//!
//! Layers:
//! - [`path::WorkspaceSandbox`] — path confinement for file tools
//! - [`process::ProcessSandboxPlan`] — optional process wrapper (macOS seatbelt)
//! - [`platform`] — capability *reporting* only (not enforcement)

pub mod path;
pub mod platform;
pub mod process;

pub use path::{SandboxError, WorkspaceSandbox};
pub use platform::{
    CapabilityStatus, PlatformCapabilityReport, PlatformReportError, PluginIsolationCapability,
    SandboxCapability, SecretProviderCapability,
};
pub use process::{ProcessSandboxMode, ProcessSandboxPlan};
