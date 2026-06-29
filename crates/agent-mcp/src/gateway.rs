use std::collections::BTreeSet;
use std::sync::Arc;

use agent_tool::{Tool, ToolDefinition, ToolError, ToolInput, ToolOutput, ToolRegistry};
use serde::{Deserialize, Serialize};

use crate::{
    McpClientSamplingCapability, McpError, McpGatewayClient, McpRoot, McpSamplingValidationPolicy,
    McpServerConfig, McpServerHealth, McpServerId, McpServerStatus, McpToolCallRequest,
    McpToolDefinition, McpToolDescriptor, check_mcp_server_health,
    mcp_descriptor_to_tool_definition,
};

const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024;
const MAX_GATEWAY_MESSAGE_CHARS: usize = 256;

#[derive(Clone)]
pub struct McpGateway {
    client: Arc<dyn McpGatewayClient>,
    max_response_bytes: usize,
}

impl McpGateway {
    pub fn new<C>(client: Arc<C>) -> Self
    where
        C: McpGatewayClient + 'static,
    {
        Self {
            client,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        }
    }

    #[must_use]
    pub const fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    pub async fn discover_tools(&self, configs: &[McpServerConfig]) -> McpGatewayReport {
        let mut servers = Vec::new();
        let mut tools = Vec::new();
        let mut roots = Vec::new();
        let mut sampling = Vec::new();
        let mut diagnostics = Vec::new();
        let mut local_names = BTreeSet::new();
        let client_capabilities = self.client.client_capabilities();
        let sampling_policy = McpSamplingValidationPolicy::default();

        for config in configs {
            let health = check_mcp_server_health(self.client.as_ref(), config).await;
            if config.enabled && health.status == McpServerStatus::Healthy {
                if let Some(sampling_capability) = &client_capabilities.sampling {
                    sampling.push(gateway_sampling_from_capability(
                        config.id.clone(),
                        sampling_capability,
                    ));
                    diagnostics.extend(gateway_sampling_policy_diagnostics(
                        config.id.clone(),
                        sampling_capability,
                        &sampling_policy,
                    ));
                }

                match self.client.list_roots(&config.id).await {
                    Ok(list) => {
                        for root in list.roots {
                            roots.push(gateway_root_from_root(config.id.clone(), root));
                        }
                    }
                    Err(error) => diagnostics.push(McpGatewayDiagnostic::error(
                        "root_discovery_failed",
                        error.to_string(),
                        Some(config.id.clone()),
                    )),
                }

                match self.client.list_tools(&config.id).await {
                    Ok(list) => {
                        for descriptor in list.tools {
                            match gateway_tool_from_descriptor(descriptor) {
                                Ok(tool) if local_names.insert(tool.local_name.clone()) => {
                                    tools.push(tool);
                                }
                                Ok(tool) => diagnostics.push(McpGatewayDiagnostic::error(
                                    "duplicate_local_tool_name",
                                    format!("duplicate MCP local tool name `{}`", tool.local_name),
                                    Some(config.id.clone()),
                                )),
                                Err(error) => diagnostics.push(McpGatewayDiagnostic::error(
                                    "invalid_tool_descriptor",
                                    error.to_string(),
                                    Some(config.id.clone()),
                                )),
                            }
                        }
                    }
                    Err(error) => diagnostics.push(McpGatewayDiagnostic::error(
                        "tool_discovery_failed",
                        error.to_string(),
                        Some(config.id.clone()),
                    )),
                }
            }
            servers.push(health);
        }

        servers.sort_by_key(|server| server.server_id.to_string());
        tools.sort_by_key(|tool| tool.local_name.clone());
        roots.sort_by(|left, right| {
            left.server_id
                .cmp(&right.server_id)
                .then_with(|| left.uri.cmp(&right.uri))
                .then_with(|| left.name.cmp(&right.name))
        });
        sampling.sort_by_key(|descriptor| descriptor.server_id.to_string());
        diagnostics.sort_by_key(|diagnostic| diagnostic.code.clone());

        McpGatewayReport {
            servers,
            tools,
            roots,
            sampling,
            diagnostics,
        }
    }

    pub fn register_tools(
        &self,
        registry: &mut ToolRegistry,
        report: &McpGatewayReport,
    ) -> Result<(), McpError> {
        for tool in &report.tools {
            registry
                .register(McpGatewayTool {
                    definition: tool.definition.clone(),
                    server_id: tool.server_id.clone(),
                    remote_tool_name: tool.remote_tool_name.clone(),
                    client: Arc::clone(&self.client),
                    max_response_bytes: self.max_response_bytes,
                })
                .map_err(|error| McpError::Client(error.to_string()))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpGatewayReport {
    pub servers: Vec<McpServerHealth>,
    pub tools: Vec<McpGatewayToolDescriptor>,
    pub roots: Vec<McpGatewayRootDescriptor>,
    pub sampling: Vec<McpGatewaySamplingDescriptor>,
    pub diagnostics: Vec<McpGatewayDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpGatewayToolDescriptor {
    pub server_id: McpServerId,
    pub remote_tool_name: String,
    pub local_name: String,
    pub definition: ToolDefinition,
    pub descriptor: McpToolDescriptor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpGatewayRootDescriptor {
    pub server_id: McpServerId,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpGatewaySamplingDescriptor {
    pub server_id: McpServerId,
    pub available: bool,
    pub tools_supported: bool,
    pub context_supported: bool,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpGatewayDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub server_id: Option<McpServerId>,
}

impl McpGatewayDiagnostic {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        server_id: Option<McpServerId>,
    ) -> Self {
        Self::new("error", code, message, server_id)
    }

    fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        server_id: Option<McpServerId>,
    ) -> Self {
        Self::new("warning", code, message, server_id)
    }

    fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        server_id: Option<McpServerId>,
    ) -> Self {
        Self {
            severity: severity.into(),
            code: code.into(),
            message: gateway_public_message(&message.into()),
            server_id,
        }
    }
}

struct McpGatewayTool {
    definition: ToolDefinition,
    server_id: McpServerId,
    remote_tool_name: String,
    client: Arc<dyn McpGatewayClient>,
    max_response_bytes: usize,
}

impl Tool for McpGatewayTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }

    fn execute(&self, input: ToolInput) -> Result<ToolOutput, ToolError> {
        let result = futures::executor::block_on(self.client.call_tool(McpToolCallRequest {
            server_id: self.server_id.clone(),
            remote_tool_name: self.remote_tool_name.clone(),
            input_json: redact_secret_like_json(input.value),
        }))
        .map_err(|error| ToolError::Execution(redact_secret_like_text(&error.to_string())))?;

        Ok(ToolOutput {
            value: bounded_tool_output(result.output_json, self.max_response_bytes),
        })
    }
}

fn gateway_tool_from_descriptor(
    descriptor: McpToolDescriptor,
) -> Result<McpGatewayToolDescriptor, McpError> {
    let mapped: McpToolDefinition = mcp_descriptor_to_tool_definition(&descriptor)?;
    Ok(McpGatewayToolDescriptor {
        server_id: descriptor.server_id.clone(),
        remote_tool_name: descriptor.name.clone(),
        local_name: mapped.local_name.as_str().to_string(),
        definition: mapped.definition,
        descriptor,
    })
}

fn gateway_root_from_root(server_id: McpServerId, root: McpRoot) -> McpGatewayRootDescriptor {
    McpGatewayRootDescriptor {
        server_id,
        uri: redact_secret_like_text(&root.uri),
        name: root.name.map(|name| redact_secret_like_text(&name)),
    }
}

fn gateway_sampling_from_capability(
    server_id: McpServerId,
    capability: &McpClientSamplingCapability,
) -> McpGatewaySamplingDescriptor {
    McpGatewaySamplingDescriptor {
        server_id,
        available: true,
        tools_supported: capability.tools,
        context_supported: capability.context,
        message: gateway_public_message("MCP sampling available with safe validation policy"),
    }
}

fn gateway_sampling_policy_diagnostics(
    server_id: McpServerId,
    capability: &McpClientSamplingCapability,
    policy: &McpSamplingValidationPolicy,
) -> Vec<McpGatewayDiagnostic> {
    let mut diagnostics = Vec::new();
    if capability.tools && !policy.allow_tools {
        diagnostics.push(McpGatewayDiagnostic::warning(
            "sampling_tools_rejected_by_policy",
            "MCP sampling tool requests are rejected by the current safe validation policy",
            Some(server_id.clone()),
        ));
    }
    if capability.context && !policy.allow_context {
        diagnostics.push(McpGatewayDiagnostic::warning(
            "sampling_context_rejected_by_policy",
            "MCP sampling context requests are rejected by the current safe validation policy",
            Some(server_id),
        ));
    }
    diagnostics
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

fn gateway_public_message(value: &str) -> String {
    truncate_chars(&redact_secret_like_text(value), MAX_GATEWAY_MESSAGE_CHARS)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }

    let prefix = value.chars().take(max_chars - 3).collect::<String>();
    format!("{prefix}...")
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

fn redact_secret_like_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => {
            serde_json::Value::String(redact_secret_like_text(&text))
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(redact_secret_like_json).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    if is_secret_like(&key) {
                        (key, serde_json::Value::String("[redacted]".to_string()))
                    } else {
                        (key, redact_secret_like_json(value))
                    }
                })
                .collect(),
        ),
        value => value,
    }
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
