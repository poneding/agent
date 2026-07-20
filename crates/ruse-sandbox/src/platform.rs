use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityStatus {
    Supported,
    Unsupported,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SandboxCapability {
    pub status: CapabilityStatus,
    pub detail: String,
}

impl SandboxCapability {
    #[must_use]
    pub fn supported(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Supported,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn unsupported(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Unsupported,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn unknown(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Unknown,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginIsolationCapability {
    pub process: SandboxCapability,
    pub wasm: SandboxCapability,
}

impl PluginIsolationCapability {
    #[must_use]
    pub fn new(process: SandboxCapability, wasm: SandboxCapability) -> Self {
        Self { process, wasm }
    }

    #[must_use]
    pub fn unknown() -> Self {
        Self {
            process: SandboxCapability::unknown("process plugin isolation unknown"),
            wasm: SandboxCapability::unknown("wasm plugin isolation unknown"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretProviderCapability {
    pub status: CapabilityStatus,
    pub provider: Option<String>,
    pub detail: String,
}

impl SecretProviderCapability {
    #[must_use]
    pub fn available(provider: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Supported,
            provider: Some(provider.into()),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Unsupported,
            provider: None,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn unknown(detail: impl Into<String>) -> Self {
        Self {
            status: CapabilityStatus::Unknown,
            provider: None,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformCapabilityReport {
    pub os: String,
    pub filesystem_sandbox: SandboxCapability,
    pub symlink_protection: SandboxCapability,
    pub process_execution: SandboxCapability,
    pub network_default: SandboxCapability,
    pub plugin_isolation: PluginIsolationCapability,
    pub secret_provider: SecretProviderCapability,
}

impl PlatformCapabilityReport {
    pub fn new(
        os: impl Into<String>,
        filesystem_sandbox: SandboxCapability,
        symlink_protection: SandboxCapability,
        process_execution: SandboxCapability,
        network_default: SandboxCapability,
        plugin_isolation: PluginIsolationCapability,
        secret_provider: SecretProviderCapability,
    ) -> Result<Self, PlatformReportError> {
        let os = os.into();
        if os.trim().is_empty() {
            return Err(PlatformReportError::EmptyOs);
        }
        Ok(Self {
            os,
            filesystem_sandbox,
            symlink_protection,
            process_execution,
            network_default,
            plugin_isolation,
            secret_provider,
        })
    }

    pub fn current() -> Result<Self, PlatformReportError> {
        Self::new(
            std::env::consts::OS,
            SandboxCapability::supported("workspace path confinement with canonicalized roots"),
            SandboxCapability::supported("symlink escapes rejected by canonical path checks"),
            SandboxCapability::supported("child process execution available"),
            SandboxCapability::unknown("network access is governed outside ruse-sandbox"),
            PluginIsolationCapability::new(
                SandboxCapability::supported("process plugin host available"),
                SandboxCapability::supported("wasm module validation prototype available"),
            ),
            SecretProviderCapability::unknown("local secret provider not configured"),
        )
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PlatformReportError {
    #[error("platform capability report os must not be empty")]
    EmptyOs,
}
