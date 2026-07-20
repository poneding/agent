use crate::error::PluginManifestError;
use crate::manifest::PluginRuntimeCompatibility;

pub const PLUGIN_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const PLUGIN_RUNTIME_API_VERSION: u32 = 1;

pub fn check_plugin_compatibility(
    compatibility: &PluginRuntimeCompatibility,
) -> Result<(), PluginManifestError> {
    if compatibility.min_agent_version.trim().is_empty() {
        return Err(PluginManifestError::IncompatibleRuntime(
            "min_agent_version cannot be empty".to_string(),
        ));
    }

    if compatibility.api_version != PLUGIN_RUNTIME_API_VERSION {
        return Err(PluginManifestError::IncompatibleRuntime(format!(
            "unsupported plugin runtime api version {}; supported version is {}",
            compatibility.api_version, PLUGIN_RUNTIME_API_VERSION
        )));
    }

    Ok(())
}
