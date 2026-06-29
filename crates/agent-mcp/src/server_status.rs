use serde::{Deserialize, Serialize};

use crate::{McpResourceExportRegistry, McpServerState, McpToolExportRegistry};

const MAX_STATUS_TEXT_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerStatusSummary {
    pub available: bool,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: String,
    pub initialized: bool,
    pub capabilities: McpServerStatusCapabilities,
    pub stdio_fixture_available: bool,
    pub exported_tool_count: usize,
    pub exported_resource_count: usize,
    pub exported_tool_names: Vec<String>,
    pub exported_resource_uris: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerStatusCapabilities {
    pub tools: bool,
    pub prompts: bool,
    pub resources: bool,
}

pub fn mcp_server_status_summary(
    state: &McpServerState,
    tool_exports: &McpToolExportRegistry,
    resource_exports: &McpResourceExportRegistry,
) -> McpServerStatusSummary {
    let exported_tool_names = tool_exports
        .list_tools()
        .into_iter()
        .map(|tool| safe_status_text(&tool.name))
        .collect::<Vec<_>>();
    let exported_resource_uris = resource_exports
        .list_resources()
        .into_iter()
        .map(|resource| safe_status_text(&resource.uri))
        .collect::<Vec<_>>();

    McpServerStatusSummary {
        available: true,
        protocol_version: safe_status_text(&state.info.protocol_version),
        server_name: safe_status_text(&state.info.name),
        server_version: safe_status_text(&state.info.version),
        initialized: state.initialized,
        capabilities: McpServerStatusCapabilities {
            tools: state.capabilities.tools,
            prompts: state.capabilities.prompts,
            resources: state.capabilities.resources,
        },
        stdio_fixture_available: true,
        exported_tool_count: exported_tool_names.len(),
        exported_resource_count: exported_resource_uris.len(),
        exported_tool_names,
        exported_resource_uris,
    }
}

fn safe_status_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    bound_chars(value, MAX_STATUS_TEXT_CHARS)
}

fn bound_chars(value: &str, max_chars: usize) -> String {
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
