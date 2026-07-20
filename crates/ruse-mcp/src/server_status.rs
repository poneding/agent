use serde::{Deserialize, Serialize};

use crate::{McpResourceExportRegistry, McpServerState, McpToolExportRegistry};

const MAX_STATUS_TEXT_CHARS: usize = 256;
const MAX_STATUS_HEALTH_DIAGNOSTICS: usize = 8;

pub const MCP_SERVER_STATUS_HEALTH_SCHEMA_VERSION: u32 = 1;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatusHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatusHealthDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerStatusHealthDiagnostic {
    pub severity: McpServerStatusHealthDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerStatusHealthReport {
    pub schema_version: u32,
    pub status: McpServerStatusHealth,
    pub diagnostics: Vec<McpServerStatusHealthDiagnostic>,
    pub next_actions: Vec<String>,
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

#[must_use]
pub fn mcp_server_status_health_report(
    summary: &McpServerStatusSummary,
) -> McpServerStatusHealthReport {
    let mut diagnostics = Vec::new();
    let mut next_actions = Vec::<String>::new();

    if !summary.available {
        diagnostics.push(mcp_status_health_diagnostic(
            McpServerStatusHealthDiagnosticSeverity::Error,
            "mcp.server.unavailable",
            "MCP server is unavailable",
        ));
    }
    if !summary.initialized {
        diagnostics.push(mcp_status_health_diagnostic(
            McpServerStatusHealthDiagnosticSeverity::Info,
            "mcp.server.not_initialized",
            "MCP server has not completed initialize",
        ));
        push_next_action(&mut next_actions, "mcp.server.initialize");
    }
    if summary.exported_tool_count == 0 && summary.exported_resource_count == 0 {
        diagnostics.push(mcp_status_health_diagnostic(
            McpServerStatusHealthDiagnosticSeverity::Warning,
            "mcp.server.no_exports",
            "MCP server has no exported tools or resources",
        ));
        push_next_action(&mut next_actions, "mcp.server.configure_exports");
    }
    if !summary.capabilities.tools && summary.exported_tool_count > 0 {
        diagnostics.push(mcp_status_health_diagnostic(
            McpServerStatusHealthDiagnosticSeverity::Error,
            "mcp.server.tool_exports_without_capability",
            "MCP server exports tools but tool capability is disabled",
        ));
        push_next_action(&mut next_actions, "mcp.server.fix_capabilities");
    }
    if !summary.capabilities.resources && summary.exported_resource_count > 0 {
        diagnostics.push(mcp_status_health_diagnostic(
            McpServerStatusHealthDiagnosticSeverity::Error,
            "mcp.server.resource_exports_without_capability",
            "MCP server exports resources but resource capability is disabled",
        ));
        push_next_action(&mut next_actions, "mcp.server.fix_capabilities");
    }

    let status = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == McpServerStatusHealthDiagnosticSeverity::Error)
    {
        McpServerStatusHealth::Unhealthy
    } else if !diagnostics.is_empty() {
        McpServerStatusHealth::Degraded
    } else {
        McpServerStatusHealth::Healthy
    };

    diagnostics.truncate(MAX_STATUS_HEALTH_DIAGNOSTICS);

    McpServerStatusHealthReport {
        schema_version: MCP_SERVER_STATUS_HEALTH_SCHEMA_VERSION,
        status,
        diagnostics,
        next_actions,
    }
}

fn mcp_status_health_diagnostic(
    severity: McpServerStatusHealthDiagnosticSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
) -> McpServerStatusHealthDiagnostic {
    McpServerStatusHealthDiagnostic {
        severity,
        code: safe_status_text(&code.into()),
        message: safe_status_text(&message.into()),
        truncated: false,
    }
}

fn push_next_action(next_actions: &mut Vec<String>, action: &str) {
    if !next_actions.iter().any(|existing| existing == action) {
        next_actions.push(action.to_string());
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
