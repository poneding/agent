use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    MCP_SAMPLING_CREATE_MESSAGE_METHOD, McpClientRootProvider, McpCreateMessageRequestParams,
    McpError, McpJsonRpcError, McpJsonRpcFailure, McpJsonRpcId, McpJsonRpcRequest,
    McpJsonRpcSuccess, McpSamplingProvider, McpServerConfig, create_mcp_sampling_message,
    list_mcp_client_roots,
};

const MAX_CALLBACK_FRAME_BYTES: usize = 64 * 1024;
const MAX_CALLBACK_DIAGNOSTIC_CHARS: usize = 256;
const MCP_ROOTS_LIST_METHOD: &str = "roots/list";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientCallbackTransportKind {
    Stdio,
    StreamableHttp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientCallbackPhase {
    ToolCallInFlight,
    StreamDrain,
    Fixture,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientCallbackKind {
    RootsList,
    SamplingCreateMessage,
    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientCallbackOutcome {
    Responded,
    Rejected,
    IgnoredNotification,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientCallbackDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientCallbackDiagnostic {
    pub severity: McpClientCallbackDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl McpClientCallbackDiagnostic {
    #[must_use]
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpClientCallbackDiagnosticSeverity::Info, code, message)
    }

    #[must_use]
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpClientCallbackDiagnosticSeverity::Warning, code, message)
    }

    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpClientCallbackDiagnosticSeverity::Error, code, message)
    }

    fn new(
        severity: McpClientCallbackDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: bound_public_callback_text(&code.into()),
            message: redact_callback_text(&message.into()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientCallbackReport {
    pub server_id: crate::McpServerId,
    pub transport_kind: McpClientCallbackTransportKind,
    pub phase: McpClientCallbackPhase,
    pub method: String,
    pub callback_kind: McpClientCallbackKind,
    pub outcome: McpClientCallbackOutcome,
    pub response_emitted: bool,
    pub diagnostics: Vec<McpClientCallbackDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpClientCallbackDispatch {
    pub report: McpClientCallbackReport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_frame: Option<String>,
}

struct CallbackDispatchContext<'a> {
    config: &'a McpServerConfig,
    transport_kind: McpClientCallbackTransportKind,
    phase: McpClientCallbackPhase,
    method: &'a str,
    callback_kind: McpClientCallbackKind,
    outcome: McpClientCallbackOutcome,
}

pub async fn handle_mcp_client_callback_frame(
    root_provider: &dyn McpClientRootProvider,
    sampling_provider: &dyn McpSamplingProvider,
    config: &McpServerConfig,
    transport_kind: McpClientCallbackTransportKind,
    phase: McpClientCallbackPhase,
    frame: &str,
) -> McpClientCallbackDispatch {
    let request = match parse_callback_request_frame(frame) {
        Ok(request) => request,
        Err(error) => {
            let response_frame = render_callback_failure(McpJsonRpcId::Null, error);
            return callback_dispatch(
                CallbackDispatchContext {
                    config,
                    transport_kind,
                    phase,
                    method: "invalid",
                    callback_kind: McpClientCallbackKind::Unknown,
                    outcome: McpClientCallbackOutcome::Rejected,
                },
                Some(response_frame),
                vec![McpClientCallbackDiagnostic::error(
                    "callback_parse_failed",
                    "MCP callback frame could not be parsed safely",
                )],
            );
        }
    };

    let method = request.method.clone();
    let callback_kind = callback_kind_for_method(&method);
    let Some(id) = request.id else {
        return callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: &method,
                callback_kind,
                outcome: McpClientCallbackOutcome::IgnoredNotification,
            },
            None,
            vec![McpClientCallbackDiagnostic::warning(
                "callback_notification_ignored",
                "MCP callback notification was ignored because callbacks require a request id",
            )],
        );
    };

    match method.as_str() {
        MCP_ROOTS_LIST_METHOD => {
            handle_roots_list_callback(root_provider, config, transport_kind, phase, id).await
        }
        MCP_SAMPLING_CREATE_MESSAGE_METHOD => {
            handle_sampling_callback(
                sampling_provider,
                config,
                transport_kind,
                phase,
                id,
                request.params,
            )
            .await
        }
        _ => callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: &method,
                callback_kind: McpClientCallbackKind::Unknown,
                outcome: McpClientCallbackOutcome::Rejected,
            },
            Some(render_callback_failure(
                id,
                McpJsonRpcError::method_not_found(
                    method.clone(),
                    "unsupported MCP client callback",
                ),
            )),
            vec![McpClientCallbackDiagnostic::warning(
                "callback_unknown_method",
                "MCP client callback method is not supported",
            )],
        ),
    }
}

async fn handle_roots_list_callback(
    root_provider: &dyn McpClientRootProvider,
    config: &McpServerConfig,
    transport_kind: McpClientCallbackTransportKind,
    phase: McpClientCallbackPhase,
    id: McpJsonRpcId,
) -> McpClientCallbackDispatch {
    match list_mcp_client_roots(root_provider, config).await {
        Ok(roots) => callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: MCP_ROOTS_LIST_METHOD,
                callback_kind: McpClientCallbackKind::RootsList,
                outcome: McpClientCallbackOutcome::Responded,
            },
            Some(render_callback_success(
                id,
                serde_json::to_value(roots).unwrap_or_else(|_| Value::Object(Default::default())),
            )),
            vec![McpClientCallbackDiagnostic::info(
                "roots_list_callback_completed",
                "MCP roots/list callback completed with safe root metadata",
            )],
        ),
        Err(error) => callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: MCP_ROOTS_LIST_METHOD,
                callback_kind: McpClientCallbackKind::RootsList,
                outcome: McpClientCallbackOutcome::Rejected,
            },
            Some(render_callback_failure(
                id,
                mcp_error_to_callback_error(error),
            )),
            vec![McpClientCallbackDiagnostic::error(
                "roots_list_callback_failed",
                "MCP roots/list callback failed at the local provider boundary",
            )],
        ),
    }
}

async fn handle_sampling_callback(
    sampling_provider: &dyn McpSamplingProvider,
    config: &McpServerConfig,
    transport_kind: McpClientCallbackTransportKind,
    phase: McpClientCallbackPhase,
    id: McpJsonRpcId,
    params: Value,
) -> McpClientCallbackDispatch {
    let request = match serde_json::from_value::<McpCreateMessageRequestParams>(params) {
        Ok(request) => request,
        Err(_) => {
            return callback_dispatch(
                CallbackDispatchContext {
                    config,
                    transport_kind,
                    phase,
                    method: MCP_SAMPLING_CREATE_MESSAGE_METHOD,
                    callback_kind: McpClientCallbackKind::SamplingCreateMessage,
                    outcome: McpClientCallbackOutcome::Rejected,
                },
                Some(render_callback_failure(
                    id,
                    McpJsonRpcError::invalid_params("invalid sampling/createMessage params"),
                )),
                vec![McpClientCallbackDiagnostic::error(
                    "sampling_callback_invalid_params",
                    "MCP sampling callback params were invalid",
                )],
            );
        }
    };

    match create_mcp_sampling_message(sampling_provider, config, request).await {
        Ok(result) => callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: MCP_SAMPLING_CREATE_MESSAGE_METHOD,
                callback_kind: McpClientCallbackKind::SamplingCreateMessage,
                outcome: McpClientCallbackOutcome::Responded,
            },
            Some(render_callback_success(
                id,
                serde_json::to_value(result).unwrap_or_else(|_| Value::Object(Default::default())),
            )),
            vec![McpClientCallbackDiagnostic::info(
                "sampling_callback_completed",
                "MCP sampling callback completed through the local sampling boundary",
            )],
        ),
        Err(error) => callback_dispatch(
            CallbackDispatchContext {
                config,
                transport_kind,
                phase,
                method: MCP_SAMPLING_CREATE_MESSAGE_METHOD,
                callback_kind: McpClientCallbackKind::SamplingCreateMessage,
                outcome: McpClientCallbackOutcome::Rejected,
            },
            Some(render_callback_failure(
                id,
                mcp_error_to_callback_error(error),
            )),
            vec![McpClientCallbackDiagnostic::error(
                "sampling_callback_rejected",
                "MCP sampling callback was rejected by local validation or provider policy",
            )],
        ),
    }
}

fn parse_callback_request_frame(frame: &str) -> Result<McpJsonRpcRequest, McpJsonRpcError> {
    if frame.len() > MAX_CALLBACK_FRAME_BYTES {
        return Err(McpJsonRpcError::invalid_request(
            "MCP callback frame too large",
        ));
    }

    let request = serde_json::from_str::<McpJsonRpcRequest>(frame.trim_end())
        .map_err(|_| McpJsonRpcError::parse_error("invalid MCP callback JSON-RPC frame"))?;
    if request.jsonrpc != "2.0" {
        return Err(McpJsonRpcError::invalid_request(
            "MCP callback JSON-RPC version must be 2.0",
        ));
    }
    Ok(request)
}

fn callback_kind_for_method(method: &str) -> McpClientCallbackKind {
    match method {
        MCP_ROOTS_LIST_METHOD => McpClientCallbackKind::RootsList,
        MCP_SAMPLING_CREATE_MESSAGE_METHOD => McpClientCallbackKind::SamplingCreateMessage,
        _ => McpClientCallbackKind::Unknown,
    }
}

fn callback_dispatch(
    context: CallbackDispatchContext<'_>,
    response_frame: Option<String>,
    diagnostics: Vec<McpClientCallbackDiagnostic>,
) -> McpClientCallbackDispatch {
    McpClientCallbackDispatch {
        report: McpClientCallbackReport {
            server_id: context.config.id.clone(),
            transport_kind: context.transport_kind,
            phase: context.phase,
            method: bound_public_callback_text(context.method),
            callback_kind: context.callback_kind,
            outcome: context.outcome,
            response_emitted: response_frame.is_some(),
            diagnostics,
        },
        response_frame,
    }
}

fn render_callback_success(id: McpJsonRpcId, result: Value) -> String {
    render_callback_frame(&McpJsonRpcSuccess::new(id, result))
}

fn render_callback_failure(id: McpJsonRpcId, error: McpJsonRpcError) -> String {
    render_callback_frame(&McpJsonRpcFailure::new(id, error))
}

fn render_callback_frame<T: Serialize>(frame: &T) -> String {
    match serde_json::to_string(frame) {
        Ok(mut rendered) => {
            rendered.push('\n');
            rendered
        }
        Err(_) => {
            "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32603,\"message\":\"serialization failed\"}}\n"
                .to_string()
        }
    }
}

fn mcp_error_to_callback_error(error: McpError) -> McpJsonRpcError {
    match error {
        McpError::InvalidServerRequest(message) if is_safe_sampling_rejection(&message) => {
            McpJsonRpcError::invalid_request(bound_public_callback_text(&message))
        }
        McpError::InvalidServerRequest(_) | McpError::InvalidConfig(_) => {
            McpJsonRpcError::invalid_request("MCP client callback rejected by local policy")
        }
        McpError::Client(_) | McpError::ServerNotFound(_) | McpError::InvalidToolDescriptor(_) => {
            McpJsonRpcError::internal_error("MCP client callback provider failed")
        }
    }
}

fn is_safe_sampling_rejection(message: &str) -> bool {
    let Some(codes) = message.strip_prefix("sampling request rejected: ") else {
        return false;
    };

    !codes.is_empty()
        && codes.split(", ").all(|code| {
            !code.is_empty() && code.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_')
        })
}

fn redact_callback_text(value: &str) -> String {
    bound_public_callback_text(&redact_secret_like_text(value))
}

fn bound_public_callback_text(value: &str) -> String {
    let mut chars = value.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_CALLBACK_DIAGNOSTIC_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn redact_secret_like_text(value: &str) -> String {
    if is_secret_like(value) {
        return "[redacted]".to_string();
    }
    value
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
        || normalized.starts_with("sk_")
}
