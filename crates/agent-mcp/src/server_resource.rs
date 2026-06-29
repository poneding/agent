use std::collections::BTreeMap;

use agent_policy::{
    CapabilityAction, PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
};
use agent_types::{ActorRef, CorrelationId};
use serde::{Deserialize, Serialize};

use crate::McpError;

const DEFAULT_MAX_RESOURCE_CONTENT_BYTES: usize = 16 * 1024;
const MAX_RESOURCE_DIAGNOSTIC_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceExport {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpResourceReadResult {
    pub uri: String,
    pub contents: String,
    pub mime_type: Option<String>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct McpResourceRecord {
    export: McpResourceExport,
    contents: String,
    sensitive: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpResourceExportRegistry {
    resources: BTreeMap<String, McpResourceRecord>,
    max_content_bytes: usize,
}

impl Default for McpResourceExportRegistry {
    fn default() -> Self {
        Self {
            resources: BTreeMap::new(),
            max_content_bytes: DEFAULT_MAX_RESOURCE_CONTENT_BYTES,
        }
    }
}

impl McpResourceExportRegistry {
    #[must_use]
    pub const fn with_max_content_bytes(mut self, max_content_bytes: usize) -> Self {
        self.max_content_bytes = max_content_bytes;
        self
    }

    pub fn export_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, true, false)
    }

    pub fn export_sensitive_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, true, true)
    }

    pub fn export_disabled_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        self.insert_resource(uri, name, mime_type, contents, false, false)
    }

    #[must_use]
    pub fn list_resources(&self) -> Vec<McpResourceExport> {
        self.resources
            .values()
            .filter(|record| record.export.enabled)
            .map(|record| record.export.clone())
            .collect()
    }

    pub fn read_resource(
        &self,
        policy: &dyn PolicyEngine,
        actor: ActorRef,
        correlation_id: CorrelationId,
        uri: &str,
    ) -> Result<McpResourceReadResult, McpError> {
        let record = self
            .resources
            .get(uri)
            .filter(|record| record.export.enabled)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP resource export not found: {}",
                    redact_text(uri)
                ))
            })?;
        let request = PermissionRequest::new(
            actor,
            CapabilityAction::Read,
            ResourceScope::new(ResourceKind::File, &record.export.uri)
                .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?,
            format!(
                "MCP server resource export requested: {}",
                record.export.uri
            ),
            correlation_id,
        )
        .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
        let decision = policy
            .decide(&request)
            .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;

        match decision {
            PolicyDecision::Allow { .. } => {
                policy
                    .issue_lease(
                        &request,
                        PolicyDecision::Allow {
                            reason: "MCP server resource export allowed".to_string(),
                        },
                    )
                    .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
            }
            PolicyDecision::Deny { reason } | PolicyDecision::Ask { reason } => {
                return Err(McpError::InvalidServerRequest(format!(
                    "policy denied MCP resource export {}: {}",
                    redact_text(&record.export.uri),
                    redact_text(&reason)
                )));
            }
        }

        let (contents, truncated) = truncate_content(&record.contents, self.max_content_bytes);
        Ok(McpResourceReadResult {
            uri: record.export.uri.clone(),
            contents,
            mime_type: record.export.mime_type.clone(),
            truncated,
        })
    }

    fn insert_resource<M>(
        &mut self,
        uri: impl Into<String>,
        name: impl Into<String>,
        mime_type: Option<M>,
        contents: impl Into<String>,
        enabled: bool,
        sensitive: bool,
    ) -> Result<(), McpError>
    where
        M: Into<String>,
    {
        let uri = validate_uri(&uri.into())?;
        if self.resources.contains_key(&uri) {
            return Err(McpError::InvalidServerRequest(format!(
                "duplicate MCP resource export: {}",
                redact_text(&uri)
            )));
        }
        let name = validate_non_empty("resource name", &name.into())?;
        let mime_type = mime_type.map(Into::into).map(|mime_type| {
            validate_non_empty("resource mime_type", &mime_type)
                .map(|value| bound_text(&value, MAX_RESOURCE_DIAGNOSTIC_CHARS))
        });
        let mime_type = match mime_type {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => return Err(error),
            None => None,
        };
        let contents = contents.into();
        if !sensitive && is_secret_like(&contents) {
            return Err(McpError::InvalidServerRequest(
                "resource content requires explicit sensitive export".to_string(),
            ));
        }

        self.resources.insert(
            uri.clone(),
            McpResourceRecord {
                export: McpResourceExport {
                    uri,
                    name,
                    mime_type,
                    enabled,
                },
                contents,
                sensitive,
            },
        );
        Ok(())
    }
}

fn validate_uri(value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().any(char::is_whitespace)
        || !trimmed.contains("://")
        || is_secret_like(trimmed)
    {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid MCP resource uri: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn validate_non_empty(field: &str, value: &str) -> Result<String, McpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || is_secret_like(trimmed) {
        return Err(McpError::InvalidServerRequest(format!(
            "invalid {field}: {}",
            redact_text(trimmed)
        )));
    }
    Ok(bound_text(trimmed, MAX_RESOURCE_DIAGNOSTIC_CHARS))
}

fn truncate_content(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_string(), false);
    }
    if max_bytes == 0 {
        return (String::new(), true);
    }
    let mut end = max_bytes.min(value.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_string(), true)
}

fn redact_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    bound_text(value, MAX_RESOURCE_DIAGNOSTIC_CHARS)
}

fn bound_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(max_chars).collect::<String>();
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
