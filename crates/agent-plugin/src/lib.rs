#![forbid(unsafe_code)]
//! Plugin runtime contracts for agent.

pub mod capability;
pub mod compatibility;
pub mod error;
pub mod hook;
pub mod host;
pub mod manifest;
pub mod process;
pub mod tool;

pub use capability::{
    PluginCapabilityKind, PluginCapabilitySet, PluginPermission,
    permission_request_for_plugin_tool, validate_plugin_capabilities,
};
pub use compatibility::{
    PLUGIN_MANIFEST_SCHEMA_VERSION, PLUGIN_RUNTIME_API_VERSION, check_plugin_compatibility,
};
pub use error::{PluginManifestError, PluginProcessError};
pub use hook::{PluginHookEvent, PluginHookOutcome, PluginHookRunner};
pub use host::PluginProcessHost;
pub use manifest::{
    PluginCapabilityManifest, PluginHookManifest, PluginIsolation, PluginIsolationMode,
    PluginManifest, PluginName, PluginRuntimeCompatibility, PluginToolManifest,
    parse_plugin_manifest_json, validate_plugin_manifest,
};
pub use process::{
    PLUGIN_PROCESS_PROTOCOL_VERSION, PluginProcessConfig, PluginProcessRequest,
    PluginProcessRequestKind, PluginProcessResponse, PluginProcessResponseKind,
    PluginProcessResult,
};
pub use tool::{
    PluginTool, PluginToolFactory, PluginToolHost, PluginToolInvocation, register_plugin_tools,
};
