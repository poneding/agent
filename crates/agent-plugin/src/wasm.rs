use serde::{Deserialize, Serialize};

use crate::error::PluginWasmError;
use crate::manifest::{PluginIsolationMode, PluginManifest};

const WASM_MAGIC: &[u8; 4] = b"\0asm";
const WASM_VERSION: u32 = 1;
const WASM_HEADER_LEN: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginWasmConfig {
    pub entrypoint: String,
    pub max_module_bytes: usize,
}

impl PluginWasmConfig {
    pub fn from_manifest(
        manifest: &PluginManifest,
        max_module_bytes: usize,
    ) -> Result<Self, PluginWasmError> {
        if manifest.isolation.mode != PluginIsolationMode::Wasm {
            return Err(PluginWasmError::InvalidConfig(
                "manifest isolation mode must be wasm".to_string(),
            ));
        }

        let entrypoint = manifest
            .isolation
            .entrypoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                PluginWasmError::InvalidConfig(
                    "wasm plugins require a non-empty entrypoint".to_string(),
                )
            })?;

        validate_max_module_bytes(max_module_bytes)?;

        Ok(Self {
            entrypoint: entrypoint.to_string(),
            max_module_bytes,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginWasmModule {
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginWasmModuleDescription {
    pub entrypoint: String,
    pub module_bytes: usize,
    pub version: u32,
}

#[derive(Clone, Debug, Default)]
pub struct PluginWasmHost;

impl PluginWasmHost {
    pub fn describe_module(
        &self,
        config: &PluginWasmConfig,
        module: &PluginWasmModule,
    ) -> Result<PluginWasmModuleDescription, PluginWasmError> {
        validate_config(config)?;
        let version = validate_module(config, module)?;

        Ok(PluginWasmModuleDescription {
            entrypoint: config.entrypoint.trim().to_string(),
            module_bytes: module.bytes.len(),
            version,
        })
    }

    pub fn call_host_function(
        &self,
        name: impl Into<String>,
    ) -> Result<serde_json::Value, PluginWasmError> {
        Err(PluginWasmError::UnsupportedHostCall { name: name.into() })
    }
}

fn validate_config(config: &PluginWasmConfig) -> Result<(), PluginWasmError> {
    if config.entrypoint.trim().is_empty() {
        return Err(PluginWasmError::InvalidConfig(
            "entrypoint cannot be empty".to_string(),
        ));
    }

    validate_max_module_bytes(config.max_module_bytes)
}

fn validate_max_module_bytes(max_module_bytes: usize) -> Result<(), PluginWasmError> {
    if max_module_bytes == 0 {
        return Err(PluginWasmError::InvalidConfig(
            "max_module_bytes must be greater than zero".to_string(),
        ));
    }

    Ok(())
}

fn validate_module(
    config: &PluginWasmConfig,
    module: &PluginWasmModule,
) -> Result<u32, PluginWasmError> {
    if module.bytes.len() > config.max_module_bytes {
        return Err(PluginWasmError::ModuleTooLarge {
            limit: config.max_module_bytes,
            actual: module.bytes.len(),
        });
    }

    if module.bytes.len() < WASM_HEADER_LEN {
        return Err(PluginWasmError::InvalidModule(
            "module is shorter than the wasm header".to_string(),
        ));
    }

    if &module.bytes[..4] != WASM_MAGIC {
        return Err(PluginWasmError::InvalidModule(
            "module magic bytes must be \\0asm".to_string(),
        ));
    }

    let version = u32::from_le_bytes([
        module.bytes[4],
        module.bytes[5],
        module.bytes[6],
        module.bytes[7],
    ]);

    if version != WASM_VERSION {
        return Err(PluginWasmError::InvalidModule(format!(
            "unsupported wasm version {version}"
        )));
    }

    Ok(version)
}
