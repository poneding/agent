use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{McpClientSamplingCapability, McpError};

pub const MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD: &str = "notifications/roots/list_changed";

const MAX_ROOT_TEXT_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRoot {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl McpRoot {
    pub fn new<N>(uri: impl Into<String>, name: Option<N>) -> Result<Self, McpError>
    where
        N: Into<String>,
    {
        Ok(Self {
            uri: validate_uri_text("MCP root URI", &uri.into())?,
            name: name
                .map(|name| validate_optional_name_text("MCP root name", &name.into()))
                .transpose()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRootListResult {
    pub roots: Vec<McpRoot>,
}

impl McpRootListResult {
    #[must_use]
    pub fn from_roots<I>(roots: I) -> Self
    where
        I: IntoIterator<Item = McpRoot>,
    {
        let mut roots = roots.into_iter().collect::<Vec<_>>();
        roots.sort_by(|left, right| {
            left.uri
                .cmp(&right.uri)
                .then_with(|| left.name.cmp(&right.name))
        });
        Self { roots }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self { roots: Vec::new() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientRootsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

impl McpClientRootsCapability {
    #[must_use]
    pub const fn new(list_changed: bool) -> Self {
        Self { list_changed }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<McpClientRootsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<McpClientSamplingCapability>,
}

impl McpClientCapabilities {
    #[must_use]
    pub const fn with_roots(roots: McpClientRootsCapability) -> Self {
        Self {
            roots: Some(roots),
            sampling: None,
        }
    }

    #[must_use]
    pub const fn with_sampling(sampling: McpClientSamplingCapability) -> Self {
        Self {
            roots: None,
            sampling: Some(sampling),
        }
    }

    #[must_use]
    pub const fn with_roots_and_sampling(
        roots: McpClientRootsCapability,
        sampling: McpClientSamplingCapability,
    ) -> Self {
        Self {
            roots: Some(roots),
            sampling: Some(sampling),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpRootListChangedNotification {
    pub jsonrpc: String,
    pub method: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpClientRootCandidate {
    pub path: PathBuf,
    pub name: Option<String>,
    pub enabled: bool,
}

impl McpClientRootCandidate {
    pub fn new<P, N>(path: P, name: Option<N>) -> Self
    where
        P: Into<PathBuf>,
        N: Into<String>,
    {
        Self {
            path: path.into(),
            name: name.map(Into::into),
            enabled: true,
        }
    }

    pub fn disabled<P, N>(path: P, name: Option<N>) -> Self
    where
        P: Into<PathBuf>,
        N: Into<String>,
    {
        Self {
            path: path.into(),
            name: name.map(Into::into),
            enabled: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpClientRootRegistry {
    workspace_roots: Vec<PathBuf>,
    entries: BTreeMap<String, McpClientRootRegistryEntry>,
}

impl McpClientRootRegistry {
    pub fn with_workspace_root(root: impl AsRef<Path>) -> Result<Self, McpError> {
        Ok(Self {
            workspace_roots: vec![canonicalize_dir(
                "MCP client workspace root",
                root.as_ref(),
            )?],
            entries: BTreeMap::new(),
        })
    }

    pub fn with_workspace_roots<I, P>(roots: I) -> Result<Self, McpError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut workspace_roots = Vec::new();
        for root in roots {
            let canonical = canonicalize_dir("MCP client workspace root", root.as_ref())?;
            if !workspace_roots.contains(&canonical) {
                workspace_roots.push(canonical);
            }
        }
        workspace_roots.sort();

        if workspace_roots.is_empty() {
            return Err(McpError::InvalidConfig(
                "MCP client root registry requires at least one workspace root".to_string(),
            ));
        }

        Ok(Self {
            workspace_roots,
            entries: BTreeMap::new(),
        })
    }

    pub fn register(&mut self, candidate: McpClientRootCandidate) -> Result<(), McpError> {
        let canonical = canonicalize_dir("MCP client root path", &candidate.path)?;
        if !self
            .workspace_roots
            .iter()
            .any(|workspace| canonical.starts_with(workspace))
        {
            return Err(McpError::InvalidConfig(format!(
                "MCP client root path is outside configured workspace: {}",
                redact_path(&candidate.path)
            )));
        }

        let uri = file_uri_for_path(&canonical)?;
        let root = McpRoot::new(uri.clone(), candidate.name)?;

        if let Some(existing) = self.entries.get_mut(&uri) {
            if existing.root.name != root.name {
                return Err(McpError::InvalidConfig(format!(
                    "duplicate MCP client root with conflicting metadata: {}",
                    redact_text(&uri)
                )));
            }
            existing.enabled |= candidate.enabled;
            return Ok(());
        }

        self.entries.insert(
            uri,
            McpClientRootRegistryEntry {
                root,
                enabled: candidate.enabled,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn list_result(&self) -> McpRootListResult {
        McpRootListResult::from_roots(
            self.entries
                .values()
                .filter(|entry| entry.enabled)
                .map(|entry| entry.root.clone()),
        )
    }

    #[must_use]
    pub fn inspection(&self) -> McpClientRootInspection {
        let roots = self
            .entries
            .values()
            .map(|entry| McpClientRootInspectionEntry {
                uri: entry.root.uri.clone(),
                name: entry.root.name.clone(),
                enabled: entry.enabled,
            })
            .collect::<Vec<_>>();
        let diagnostics = self
            .entries
            .values()
            .filter(|entry| !entry.enabled)
            .map(|entry| {
                McpClientRootInspectionDiagnostic::warning(
                    "disabled_root",
                    format!("MCP client root is disabled: {}", entry.root.uri),
                    Some(entry.root.uri.clone()),
                )
            })
            .collect();

        McpClientRootInspection { roots, diagnostics }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.values().all(|entry| !entry.enabled)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct McpClientRootRegistryEntry {
    root: McpRoot,
    enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientRootInspection {
    pub roots: Vec<McpClientRootInspectionEntry>,
    pub diagnostics: Vec<McpClientRootInspectionDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientRootInspectionEntry {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientRootInspectionDiagnostic {
    pub severity: McpClientRootInspectionSeverity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl McpClientRootInspectionDiagnostic {
    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        uri: Option<String>,
    ) -> Self {
        Self {
            severity: McpClientRootInspectionSeverity::Warning,
            code: code.into(),
            message: redact_text(&message.into()),
            uri: uri.map(|value| redact_text(&value)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientRootInspectionSeverity {
    Warning,
}

impl McpRootListChangedNotification {
    #[must_use]
    pub fn new() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: MCP_ROOTS_LIST_CHANGED_NOTIFICATION_METHOD.to_string(),
        }
    }
}

impl Default for McpRootListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_uri_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn canonicalize_dir(field: &str, path: &Path) -> Result<PathBuf, McpError> {
    if is_secret_like_path(path) {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_path(path)
        )));
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| McpError::InvalidConfig(format!("invalid {field}: {}", redact_path(path))))?;
    if is_secret_like_path(&canonical) || !canonical.is_dir() {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_path(&canonical)
        )));
    }
    Ok(canonical)
}

fn file_uri_for_path(path: &Path) -> Result<String, McpError> {
    let rendered = path.to_string_lossy().replace('\\', "/");
    if rendered.trim().is_empty() || is_secret_like(&rendered) {
        return Err(McpError::InvalidConfig(format!(
            "invalid MCP client root path: {}",
            redact_text(&rendered)
        )));
    }
    if rendered.starts_with('/') {
        Ok(format!("file://{rendered}"))
    } else {
        Ok(format!("file:///{rendered}"))
    }
}

fn is_secret_like_path(path: &Path) -> bool {
    is_secret_like(&path.to_string_lossy())
}

fn redact_path(path: &Path) -> String {
    redact_text(&path.to_string_lossy())
}

fn validate_optional_name_text(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidConfig(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed))
}

fn redact_text(value: &str) -> String {
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_text(value)
}

fn bound_text(value: &str) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(MAX_ROOT_TEXT_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn is_secret_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("sk_")
}
