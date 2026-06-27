use std::collections::BTreeMap;

use agent_policy::{CapabilityAction, ResourceKind};
use agent_types::Timestamp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExtensionId(String);

impl ExtensionId {
    pub fn new(value: impl Into<String>) -> Result<Self, ExtensionRegistryError> {
        let value = value.into();
        let value = value.trim();
        if value.is_empty() {
            return Err(ExtensionRegistryError::InvalidField(
                "extension id cannot be empty".to_string(),
            ));
        }
        if !value.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '-' | '_')
        }) {
            return Err(ExtensionRegistryError::InvalidField(format!(
                "extension id must use lowercase ascii letters, digits, '.', '-', or '_': {value}"
            )));
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    Plugin,
    SkillRoot,
    McpServer,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionSourceKind {
    LocalPath,
    Config,
    BuiltIn,
    Remote,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionSource {
    pub kind: ExtensionSourceKind,
    pub reference: String,
    pub display_label: String,
    pub content_hash: Option<String>,
}

impl ExtensionSource {
    pub fn new(
        kind: ExtensionSourceKind,
        reference: impl Into<String>,
        display_label: impl Into<String>,
        content_hash: Option<impl Into<String>>,
    ) -> Result<Self, ExtensionRegistryError> {
        Ok(Self {
            kind,
            reference: redact_secret_like_text(&non_empty(
                reference,
                "extension source reference",
            )?),
            display_label: redact_secret_like_text(&non_empty(
                display_label,
                "extension source display label",
            )?),
            content_hash: content_hash
                .map(|value| normalize_optional(value.into()))
                .transpose()?
                .flatten(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ExtensionCompatibility {
    Supported {
        api_version: u32,
    },
    Unsupported {
        found_api_version: u32,
        supported_api_version: u32,
        reason: String,
    },
    Unknown {
        reason: String,
    },
}

impl ExtensionCompatibility {
    #[must_use]
    pub const fn supported(api_version: u32) -> Self {
        Self::Supported { api_version }
    }

    pub fn unsupported(
        found_api_version: u32,
        supported_api_version: u32,
        reason: impl Into<String>,
    ) -> Self {
        Self::Unsupported {
            found_api_version,
            supported_api_version,
            reason: redact_secret_like_text(&reason.into()),
        }
    }

    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: redact_secret_like_text(&reason.into()),
        }
    }

    #[must_use]
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::Supported { .. })
    }

    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Supported { api_version } => format!("supported api version {api_version}"),
            Self::Unsupported {
                found_api_version,
                supported_api_version,
                reason,
            } => format!(
                "unsupported api version {found_api_version}; supported version is {supported_api_version}: {reason}"
            ),
            Self::Unknown { reason } => format!("unknown compatibility: {reason}"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Disabled,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionHealth {
    pub status: ExtensionHealthStatus,
    pub message: String,
}

impl ExtensionHealth {
    pub fn new(
        status: ExtensionHealthStatus,
        message: impl Into<String>,
    ) -> Result<Self, ExtensionRegistryError> {
        Ok(Self {
            status,
            message: redact_secret_like_text(&non_empty(message, "extension health message")?),
        })
    }

    #[must_use]
    pub fn healthy(message: impl Into<String>) -> Self {
        Self::new(ExtensionHealthStatus::Healthy, message)
            .expect("healthy extension message is valid")
    }

    #[must_use]
    pub fn degraded(message: impl Into<String>) -> Self {
        Self::new(ExtensionHealthStatus::Degraded, message)
            .expect("degraded extension message is valid")
    }

    #[must_use]
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self::new(ExtensionHealthStatus::Unhealthy, message)
            .expect("unhealthy extension message is valid")
    }

    #[must_use]
    pub fn disabled(message: impl Into<String>) -> Self {
        Self::new(ExtensionHealthStatus::Disabled, message)
            .expect("disabled extension message is valid")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionPermissionSummary {
    pub resource_kind: ResourceKind,
    pub action: CapabilityAction,
    pub resource: String,
}

impl ExtensionPermissionSummary {
    pub fn new(
        resource_kind: ResourceKind,
        action: CapabilityAction,
        resource: impl Into<String>,
    ) -> Result<Self, ExtensionRegistryError> {
        Ok(Self {
            resource_kind,
            action,
            resource: redact_secret_like_text(&non_empty(
                resource,
                "extension permission resource",
            )?),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionCapabilitySummary {
    pub kind: String,
    pub name: String,
    pub permissions: Vec<ExtensionPermissionSummary>,
}

impl ExtensionCapabilitySummary {
    pub fn new(
        kind: impl Into<String>,
        name: impl Into<String>,
        permissions: Vec<ExtensionPermissionSummary>,
    ) -> Result<Self, ExtensionRegistryError> {
        Ok(Self {
            kind: redact_secret_like_text(&non_empty(kind, "extension capability kind")?),
            name: redact_secret_like_text(&non_empty(name, "extension capability name")?),
            permissions,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionCapabilityLease {
    pub lease_id: String,
    pub capability_name: String,
    pub resource_kind: ResourceKind,
    pub action: CapabilityAction,
    pub resource: String,
    pub expires_at: Option<Timestamp>,
}

impl ExtensionCapabilityLease {
    pub fn new(
        lease_id: impl Into<String>,
        capability_name: impl Into<String>,
        resource_kind: ResourceKind,
        action: CapabilityAction,
        resource: impl Into<String>,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, ExtensionRegistryError> {
        Ok(Self {
            lease_id: redact_secret_like_text(&non_empty(lease_id, "extension lease id")?),
            capability_name: redact_secret_like_text(&non_empty(
                capability_name,
                "extension lease capability name",
            )?),
            resource_kind,
            action,
            resource: redact_secret_like_text(&non_empty(resource, "extension lease resource")?),
            expires_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionRecord {
    pub id: ExtensionId,
    pub kind: ExtensionKind,
    pub source: ExtensionSource,
    pub enabled: bool,
    pub version: Option<String>,
    pub compatibility: ExtensionCompatibility,
    pub declared_capabilities: Vec<ExtensionCapabilitySummary>,
    pub granted_leases: Vec<ExtensionCapabilityLease>,
    pub health: ExtensionHealth,
}

impl ExtensionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ExtensionId,
        kind: ExtensionKind,
        source: ExtensionSource,
        enabled: bool,
        version: Option<String>,
        compatibility: ExtensionCompatibility,
        declared_capabilities: Vec<ExtensionCapabilitySummary>,
        granted_leases: Vec<ExtensionCapabilityLease>,
        health: ExtensionHealth,
    ) -> Result<Self, ExtensionRegistryError> {
        let mut record = Self {
            id,
            kind,
            source,
            enabled,
            version: version.map(normalize_optional).transpose()?.flatten(),
            compatibility,
            declared_capabilities,
            granted_leases,
            health,
        };

        if !record.compatibility.is_supported() {
            record.enabled = false;
            record.granted_leases.clear();
            record.health = ExtensionHealth::unhealthy(record.compatibility.summary());
        }

        Ok(record)
    }

    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self.granted_leases.clear();
        self.health = ExtensionHealth::disabled("extension disabled");
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExtensionRegistrySnapshot {
    pub records: Vec<ExtensionRecord>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtensionRegistry {
    records: BTreeMap<ExtensionId, ExtensionRecord>,
}

impl ExtensionRegistry {
    pub fn new(records: Vec<ExtensionRecord>) -> Result<Self, ExtensionRegistryError> {
        let mut registry = Self::default();
        for record in records {
            if registry.records.contains_key(&record.id) {
                return Err(ExtensionRegistryError::DuplicateId { id: record.id });
            }
            registry.records.insert(record.id.clone(), record);
        }
        Ok(registry)
    }

    pub fn upsert(&mut self, record: ExtensionRecord) -> Result<(), ExtensionRegistryError> {
        self.records.insert(record.id.clone(), record);
        Ok(())
    }

    pub fn get(&self, id: &ExtensionId) -> Option<&ExtensionRecord> {
        self.records.get(id)
    }

    #[must_use]
    pub fn list(&self) -> Vec<ExtensionRecord> {
        self.records.values().cloned().collect()
    }

    pub fn disable(&mut self, id: &ExtensionId) -> Result<ExtensionRecord, ExtensionRegistryError> {
        let record = self
            .records
            .get_mut(id)
            .ok_or_else(|| ExtensionRegistryError::NotFound { id: id.clone() })?;
        *record = record.clone().disabled();
        Ok(record.clone())
    }

    pub fn remove(&mut self, id: &ExtensionId) -> Result<ExtensionRecord, ExtensionRegistryError> {
        self.records
            .remove(id)
            .ok_or_else(|| ExtensionRegistryError::NotFound { id: id.clone() })
    }

    #[must_use]
    pub fn snapshot(&self) -> ExtensionRegistrySnapshot {
        ExtensionRegistrySnapshot {
            records: self.list(),
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ExtensionRegistryError {
    #[error("invalid extension field: {0}")]
    InvalidField(String),
    #[error("duplicate extension id: {id}")]
    DuplicateId { id: ExtensionId },
    #[error("extension not found: {id}")]
    NotFound { id: ExtensionId },
}

impl std::fmt::Display for ExtensionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn non_empty(
    value: impl Into<String>,
    label: &'static str,
) -> Result<String, ExtensionRegistryError> {
    let value = value.into();
    let value = value.trim();
    if value.is_empty() {
        return Err(ExtensionRegistryError::InvalidField(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value.to_string())
}

fn normalize_optional(value: String) -> Result<Option<String>, ExtensionRegistryError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ExtensionRegistryError::InvalidField(
            "optional extension field cannot be empty when present".to_string(),
        ));
    }
    Ok(Some(redact_secret_like_text(value)))
}

fn redact_secret_like_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(|token| {
            if is_secret_like(token) {
                "[redacted]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
}
