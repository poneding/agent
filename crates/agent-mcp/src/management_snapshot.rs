use serde::{Deserialize, Serialize};

use crate::{
    McpClientHttpAuthorizationReport, McpClientHttpStreamSupervisorReport,
    McpClientHttpStreamSupervisorStatus, McpClientRootInspectionEntry, McpGatewayDiagnostic,
    McpGatewayReport, McpServerConfig, McpServerHealth, McpServerStatus, McpTransportConfig,
};

const MAX_MCP_MANAGEMENT_TEXT_CHARS: usize = 256;
const MAX_MCP_MANAGEMENT_DIAGNOSTICS: usize = 8;
const MAX_MCP_MANAGEMENT_ROOTS: usize = 16;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpManagementTransportKind {
    Stdio,
    StreamableHttp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpManagementStreamStatus {
    NotApplicable,
    Idle,
    Running,
    WaitingRetry,
    Cancelled,
    Failed,
    Completed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpManagementAuthStatus {
    NotApplicable,
    NotConfigured,
    Ready,
    NeedsAttention,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpManagementExtensionHealth {
    Unknown,
    Healthy,
    NeedsAttention,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpManagementDiagnosticSnapshot {
    pub severity: String,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpManagementRootSnapshot {
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpManagementSamplingSnapshot {
    pub available: bool,
    pub tools_supported: bool,
    pub context_supported: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpManagementServerSnapshot {
    pub server_id: crate::McpServerId,
    pub display_name: String,
    pub enabled: bool,
    pub status: McpServerStatus,
    pub transport_kind: McpManagementTransportKind,
    pub roots: Vec<McpManagementRootSnapshot>,
    pub sampling: McpManagementSamplingSnapshot,
    pub stream_status: McpManagementStreamStatus,
    pub auth_status: McpManagementAuthStatus,
    pub last_diagnostics: Vec<McpManagementDiagnosticSnapshot>,
    pub extension_health: McpManagementExtensionHealth,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpManagementSnapshot {
    pub servers: Vec<McpManagementServerSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct McpManagementServerSnapshotInput {
    pub stream_status: Option<McpManagementStreamStatus>,
    pub auth_status: Option<McpManagementAuthStatus>,
    pub extension_health: Option<McpManagementExtensionHealth>,
    pub extra_diagnostics: Vec<McpManagementDiagnosticSnapshot>,
}

impl McpManagementSnapshot {
    #[must_use]
    pub fn from_gateway_report(configs: &[McpServerConfig], report: &McpGatewayReport) -> Self {
        Self::from_gateway_report_with_inputs(configs, report, &[])
    }

    #[must_use]
    pub fn from_gateway_report_with_inputs(
        configs: &[McpServerConfig],
        report: &McpGatewayReport,
        inputs: &[(crate::McpServerId, McpManagementServerSnapshotInput)],
    ) -> Self {
        let mut servers = configs
            .iter()
            .map(|config| {
                let health = report
                    .servers
                    .iter()
                    .find(|health| health.server_id == config.id)
                    .cloned()
                    .unwrap_or_else(|| McpServerHealth {
                        server_id: config.id.clone(),
                        status: if config.enabled {
                            McpServerStatus::NotInitialized
                        } else {
                            McpServerStatus::Disabled
                        },
                        message: "MCP server health not reported".to_string(),
                    });
                let input = inputs
                    .iter()
                    .find(|(server_id, _)| server_id == &config.id)
                    .map(|(_, input)| input);
                server_snapshot(config, &health, report, input)
            })
            .collect::<Vec<_>>();
        servers.sort_by_key(|server| server.server_id.to_string());
        Self { servers }
    }
}

impl McpManagementServerSnapshotInput {
    #[must_use]
    pub fn with_stream_report(mut self, report: &McpClientHttpStreamSupervisorReport) -> Self {
        self.stream_status = Some(match report.status {
            McpClientHttpStreamSupervisorStatus::Idle => McpManagementStreamStatus::Idle,
            McpClientHttpStreamSupervisorStatus::Running => McpManagementStreamStatus::Running,
            McpClientHttpStreamSupervisorStatus::WaitingRetry => {
                McpManagementStreamStatus::WaitingRetry
            }
            McpClientHttpStreamSupervisorStatus::Cancelled => McpManagementStreamStatus::Cancelled,
            McpClientHttpStreamSupervisorStatus::Failed => McpManagementStreamStatus::Failed,
            McpClientHttpStreamSupervisorStatus::Completed => McpManagementStreamStatus::Completed,
        });
        self
    }

    #[must_use]
    pub fn with_auth_report(mut self, report: &McpClientHttpAuthorizationReport) -> Self {
        self.auth_status = Some(if report.authorized {
            McpManagementAuthStatus::Ready
        } else if report.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.severity,
                crate::McpClientHttpAuthorizationDiagnosticSeverity::Error
            )
        }) {
            McpManagementAuthStatus::NeedsAttention
        } else {
            McpManagementAuthStatus::NotConfigured
        });
        self
    }

    #[must_use]
    pub fn with_extension_health(mut self, health: McpManagementExtensionHealth) -> Self {
        self.extension_health = Some(health);
        self
    }

    #[must_use]
    pub fn with_extra_diagnostic(mut self, diagnostic: McpManagementDiagnosticSnapshot) -> Self {
        self.extra_diagnostics.push(diagnostic);
        self
    }
}

impl McpManagementDiagnosticSnapshot {
    #[must_use]
    pub fn new(
        severity: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: redact_management_text(&severity.into()),
            code: redact_management_text(&code.into()),
            message: redact_management_text(&message.into()),
        }
    }
}

fn server_snapshot(
    config: &McpServerConfig,
    health: &McpServerHealth,
    report: &McpGatewayReport,
    input: Option<&McpManagementServerSnapshotInput>,
) -> McpManagementServerSnapshot {
    let mut diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.server_id.as_ref() == Some(&config.id))
        .map(diagnostic_from_gateway)
        .collect::<Vec<_>>();
    diagnostics.push(McpManagementDiagnosticSnapshot::new(
        "info",
        "server_health",
        &health.message,
    ));
    if let Some(input) = input {
        diagnostics.extend(input.extra_diagnostics.clone());
    }
    diagnostics.truncate(MAX_MCP_MANAGEMENT_DIAGNOSTICS);

    McpManagementServerSnapshot {
        server_id: config.id.clone(),
        display_name: redact_management_text(&config.display_name),
        enabled: config.enabled,
        status: health.status,
        transport_kind: transport_kind(&config.transport),
        roots: report
            .roots
            .iter()
            .filter(|root| root.server_id == config.id)
            .take(MAX_MCP_MANAGEMENT_ROOTS)
            .map(|root| McpManagementRootSnapshot {
                uri: redact_management_text(&root.uri),
                name: root.name.as_deref().map(redact_management_text),
            })
            .collect(),
        sampling: report
            .sampling
            .iter()
            .find(|sampling| sampling.server_id == config.id)
            .map(|sampling| McpManagementSamplingSnapshot {
                available: sampling.available,
                tools_supported: sampling.tools_supported,
                context_supported: sampling.context_supported,
            })
            .unwrap_or(McpManagementSamplingSnapshot {
                available: false,
                tools_supported: false,
                context_supported: false,
            }),
        stream_status: input
            .and_then(|input| input.stream_status)
            .unwrap_or_else(|| default_stream_status(&config.transport)),
        auth_status: input
            .and_then(|input| input.auth_status)
            .unwrap_or_else(|| default_auth_status(&config.transport)),
        last_diagnostics: diagnostics,
        extension_health: input
            .and_then(|input| input.extension_health)
            .unwrap_or(McpManagementExtensionHealth::Unknown),
    }
}

fn diagnostic_from_gateway(diagnostic: &McpGatewayDiagnostic) -> McpManagementDiagnosticSnapshot {
    McpManagementDiagnosticSnapshot::new(
        &diagnostic.severity,
        &diagnostic.code,
        &diagnostic.message,
    )
}

fn transport_kind(transport: &McpTransportConfig) -> McpManagementTransportKind {
    match transport {
        McpTransportConfig::Stdio { .. } => McpManagementTransportKind::Stdio,
        McpTransportConfig::StreamableHttp { .. } => McpManagementTransportKind::StreamableHttp,
    }
}

fn default_stream_status(transport: &McpTransportConfig) -> McpManagementStreamStatus {
    match transport {
        McpTransportConfig::Stdio { .. } => McpManagementStreamStatus::NotApplicable,
        McpTransportConfig::StreamableHttp { .. } => McpManagementStreamStatus::Idle,
    }
}

fn default_auth_status(transport: &McpTransportConfig) -> McpManagementAuthStatus {
    match transport {
        McpTransportConfig::Stdio { .. } => McpManagementAuthStatus::NotApplicable,
        McpTransportConfig::StreamableHttp { headers, .. } => {
            if headers
                .keys()
                .any(|key| key.eq_ignore_ascii_case("authorization"))
            {
                McpManagementAuthStatus::Ready
            } else {
                McpManagementAuthStatus::NotConfigured
            }
        }
    }
}

#[must_use]
pub fn mcp_management_root_from_inspection(
    entry: &McpClientRootInspectionEntry,
) -> McpManagementRootSnapshot {
    McpManagementRootSnapshot {
        uri: redact_management_text(&entry.uri),
        name: entry.name.as_deref().map(redact_management_text),
    }
}

fn redact_management_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    let mut chars = value.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_MCP_MANAGEMENT_TEXT_CHARS)
        .collect::<String>();
    let bounded = if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    };
    bounded
        .split_whitespace()
        .map(|part| {
            if is_secret_like(part) {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        || normalized.contains("authorization")
        || normalized.contains("bearer")
        || normalized.starts_with("sk_")
}
