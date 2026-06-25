#![forbid(unsafe_code)]
//! Plugin runtime contracts for agent.

pub mod compatibility;
pub mod error;
pub mod manifest;

pub use compatibility::{
    PLUGIN_MANIFEST_SCHEMA_VERSION, PLUGIN_RUNTIME_API_VERSION, check_plugin_compatibility,
};
pub use error::PluginManifestError;
pub use manifest::{
    PluginCapabilityManifest, PluginHookManifest, PluginIsolation, PluginIsolationMode,
    PluginManifest, PluginName, PluginRuntimeCompatibility, PluginToolManifest,
    parse_plugin_manifest_json, validate_plugin_manifest,
};
