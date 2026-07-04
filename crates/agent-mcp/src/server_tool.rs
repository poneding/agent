use std::collections::BTreeMap;

use agent_policy::{
    CapabilityAction, PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
};
use agent_tool::{ToolDefinition, ToolInput, ToolName, ToolRegistry};
use agent_types::{ActorRef, CorrelationId};
use serde::{Deserialize, Serialize};

use crate::{McpError, McpToolCallResult};

const DEFAULT_MAX_TOOL_RESPONSE_BYTES: usize = 16 * 1024;
const MAX_TOOL_DIAGNOSTIC_CHARS: usize = 256;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolExport {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub enabled: bool,
}

pub fn exported_tool_from_definition(
    definition: ToolDefinition,
    enabled: bool,
) -> Result<McpToolExport, McpError> {
    let name = definition.name.as_str().to_string();
    if definition.description.trim().is_empty() {
        return Err(McpError::InvalidToolDescriptor(
            "exported tool description cannot be empty".to_string(),
        ));
    }
    Ok(McpToolExport {
        name,
        description: bound_text(&definition.description, MAX_TOOL_DIAGNOSTIC_CHARS),
        input_schema: definition.input_schema,
        enabled,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpToolExportRegistry {
    exports: BTreeMap<String, McpToolExport>,
    max_response_bytes: usize,
}

impl Default for McpToolExportRegistry {
    fn default() -> Self {
        Self {
            exports: BTreeMap::new(),
            max_response_bytes: DEFAULT_MAX_TOOL_RESPONSE_BYTES,
        }
    }
}

impl McpToolExportRegistry {
    #[must_use]
    pub const fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    pub fn export_tool(&mut self, definition: ToolDefinition) -> Result<(), McpError> {
        self.insert_export(exported_tool_from_definition(definition, true)?)
    }

    pub fn export_disabled_tool(&mut self, definition: ToolDefinition) -> Result<(), McpError> {
        self.insert_export(exported_tool_from_definition(definition, false)?)
    }

    #[must_use]
    pub fn list_tools(&self) -> Vec<McpToolExport> {
        self.exports
            .values()
            .filter(|export| export.enabled)
            .cloned()
            .collect()
    }

    pub fn call_tool(
        &self,
        tools: &ToolRegistry,
        policy: &dyn PolicyEngine,
        actor: ActorRef,
        correlation_id: CorrelationId,
        tool_name: &str,
        input_json: serde_json::Value,
    ) -> Result<McpToolCallResult, McpError> {
        let export = self
            .exports
            .get(tool_name)
            .filter(|export| export.enabled)
            .ok_or_else(|| {
                McpError::InvalidServerRequest(format!(
                    "MCP tool export not found: {}",
                    redact_text(tool_name)
                ))
            })?;
        let local_name = ToolName::new(export.name.clone())
            .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
        let tool = tools.get(&local_name).ok_or_else(|| {
            McpError::InvalidServerRequest(format!(
                "local tool not registered for MCP export: {}",
                redact_text(local_name.as_str())
            ))
        })?;
        let request = PermissionRequest::new(
            actor,
            CapabilityAction::Execute,
            ResourceScope::new(ResourceKind::Tool, local_name.as_str())
                .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?,
            format!("MCP server tool export requested: {}", local_name.as_str()),
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
                            reason: "MCP server export allowed".to_string(),
                        },
                    )
                    .map_err(|error| McpError::InvalidServerRequest(error.to_string()))?;
            }
            PolicyDecision::Deny { reason } | PolicyDecision::Ask { reason } => {
                return Err(McpError::InvalidServerRequest(format!(
                    "policy denied MCP tool export {}: {}",
                    redact_text(local_name.as_str()),
                    redact_text(&reason)
                )));
            }
        }

        let output = tool
            .execute(ToolInput { value: input_json })
            .map_err(|error| McpError::InvalidServerRequest(redact_text(&error.to_string())))?;

        Ok(McpToolCallResult::new(bounded_tool_output(
            output.value,
            self.max_response_bytes,
        )))
    }

    fn insert_export(&mut self, export: McpToolExport) -> Result<(), McpError> {
        if self.exports.contains_key(&export.name) {
            return Err(McpError::InvalidToolDescriptor(format!(
                "duplicate MCP tool export: {}",
                redact_text(&export.name)
            )));
        }
        self.exports.insert(export.name.clone(), export);
        Ok(())
    }
}

fn bounded_tool_output(output: serde_json::Value, max_bytes: usize) -> serde_json::Value {
    let rendered = output.to_string();
    if rendered.len() <= max_bytes {
        return output;
    }

    serde_json::json!({
        "content": truncate_utf8(&rendered, max_bytes),
        "truncated": true,
    })
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if max_bytes == 0 {
        return String::new();
    }
    let mut end = max_bytes.min(value.len());
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn redact_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    bound_text(value, MAX_TOOL_DIAGNOSTIC_CHARS)
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
