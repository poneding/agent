use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{McpJsonRpcRequest, McpServerConfig};

const MAX_PROTOCOL_EVENT_FRAME_BYTES: usize = 64 * 1024;
const MAX_PROTOCOL_EVENT_TEXT_CHARS: usize = 256;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpProtocolEventKind {
    Progress,
    Logging,
    Task,
    Elicitation,
    Unknown,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpProtocolEventStatus {
    Accepted,
    Rejected,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpProtocolEventDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpProtocolEventDiagnostic {
    pub severity: McpProtocolEventDiagnosticSeverity,
    pub code: String,
    pub message: String,
}

impl McpProtocolEventDiagnostic {
    #[must_use]
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpProtocolEventDiagnosticSeverity::Info, code, message)
    }

    #[must_use]
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpProtocolEventDiagnosticSeverity::Warning, code, message)
    }

    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(McpProtocolEventDiagnosticSeverity::Error, code, message)
    }

    fn new(
        severity: McpProtocolEventDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: bound_protocol_event_text(&code.into()),
            message: redact_protocol_event_text(&message.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpProtocolProgressSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    pub progress_token_present: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpProtocolEventSummary {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<McpProtocolProgressSummary>,
    pub field_count: usize,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpProtocolEventReport {
    pub server_id: crate::McpServerId,
    pub method: String,
    pub event_kind: McpProtocolEventKind,
    pub status: McpProtocolEventStatus,
    pub request_id_present: bool,
    pub ui_summary: McpProtocolEventSummary,
    pub diagnostics: Vec<McpProtocolEventDiagnostic>,
}

#[must_use]
pub fn parse_mcp_protocol_event_frame(
    config: &McpServerConfig,
    frame: &str,
) -> McpProtocolEventReport {
    if frame.len() > MAX_PROTOCOL_EVENT_FRAME_BYTES {
        return rejected_protocol_event_report(
            config,
            "invalid",
            McpProtocolEventKind::Unknown,
            "event_frame_too_large",
            "MCP protocol event frame was too large",
        );
    }

    let request = match serde_json::from_str::<McpJsonRpcRequest>(frame.trim_end()) {
        Ok(request) if request.jsonrpc == "2.0" => request,
        _ => {
            return rejected_protocol_event_report(
                config,
                "invalid",
                McpProtocolEventKind::Unknown,
                "event_parse_failed",
                "MCP protocol event frame could not be parsed",
            );
        }
    };

    let event_kind = classify_protocol_event_method(&request.method);
    let params = request.params;
    let summary = build_protocol_event_summary(event_kind, &request.method, &params);
    let diagnostic = if event_kind == McpProtocolEventKind::Unknown {
        McpProtocolEventDiagnostic::warning(
            "event_unknown_method",
            "MCP protocol event method was normalized as unknown",
        )
    } else {
        McpProtocolEventDiagnostic::info(
            "event_normalized",
            "MCP protocol event was normalized into a bridge-safe summary",
        )
    };

    McpProtocolEventReport {
        server_id: config.id.clone(),
        method: bound_protocol_event_text(&request.method),
        event_kind,
        status: McpProtocolEventStatus::Accepted,
        request_id_present: request.id.is_some(),
        ui_summary: summary,
        diagnostics: vec![diagnostic],
    }
}

fn rejected_protocol_event_report(
    config: &McpServerConfig,
    method: &str,
    event_kind: McpProtocolEventKind,
    code: &str,
    message: &str,
) -> McpProtocolEventReport {
    McpProtocolEventReport {
        server_id: config.id.clone(),
        method: method.to_string(),
        event_kind,
        status: McpProtocolEventStatus::Rejected,
        request_id_present: false,
        ui_summary: McpProtocolEventSummary {
            title: "MCP event rejected".to_string(),
            message: None,
            level: None,
            status: None,
            task_ref: None,
            progress: None,
            field_count: 0,
            truncated: false,
            redacted: false,
        },
        diagnostics: vec![McpProtocolEventDiagnostic::error(code, message)],
    }
}

fn classify_protocol_event_method(method: &str) -> McpProtocolEventKind {
    match method {
        "notifications/progress" | "progress" => McpProtocolEventKind::Progress,
        "notifications/message" | "logging/message" | "notifications/logging" => {
            McpProtocolEventKind::Logging
        }
        "elicitation/create" | "elicitation/request" | "notifications/elicitation" => {
            McpProtocolEventKind::Elicitation
        }
        _ if method.starts_with("tasks/")
            || method.starts_with("task/")
            || method.starts_with("notifications/tasks/")
            || method.starts_with("notifications/task/") =>
        {
            McpProtocolEventKind::Task
        }
        _ => McpProtocolEventKind::Unknown,
    }
}

fn build_protocol_event_summary(
    event_kind: McpProtocolEventKind,
    method: &str,
    params: &Value,
) -> McpProtocolEventSummary {
    let title = match event_kind {
        McpProtocolEventKind::Progress => "MCP progress event",
        McpProtocolEventKind::Logging => "MCP logging event",
        McpProtocolEventKind::Task => "MCP task event",
        McpProtocolEventKind::Elicitation => "MCP elicitation event",
        McpProtocolEventKind::Unknown => "MCP protocol event",
    };
    let field_count = params.as_object().map_or(0, serde_json::Map::len);
    let message = extract_safe_string(params, &["message", "statusMessage", "text"]);
    let level = extract_safe_string(params, &["level", "severity"]);
    let status = extract_safe_string(params, &["status", "state"]);
    let task_ref = extract_safe_string(params, &["taskId", "task_id", "id"]);
    let progress =
        (event_kind == McpProtocolEventKind::Progress).then(|| McpProtocolProgressSummary {
            progress: extract_number(params, "progress"),
            total: extract_number(params, "total"),
            progress_token_present: nested_field_present(params, "progressToken")
                || nested_field_present(params, "progress_token"),
        });
    let redacted = value_contains_secret_like(params)
        || message
            .as_deref()
            .is_some_and(|value| value.contains("[redacted]"))
        || level
            .as_deref()
            .is_some_and(|value| value.contains("[redacted]"))
        || status
            .as_deref()
            .is_some_and(|value| value.contains("[redacted]"))
        || task_ref
            .as_deref()
            .is_some_and(|value| value.contains("[redacted]"));

    McpProtocolEventSummary {
        title: title.to_string(),
        message,
        level,
        status,
        task_ref: task_ref.filter(|_| event_kind == McpProtocolEventKind::Task),
        progress,
        field_count,
        truncated: rendered_value_len(params) > MAX_PROTOCOL_EVENT_TEXT_CHARS,
        redacted,
    }
    .with_unknown_method_note(event_kind, method)
}

trait UnknownMethodNote {
    fn with_unknown_method_note(self, event_kind: McpProtocolEventKind, method: &str) -> Self;
}

impl UnknownMethodNote for McpProtocolEventSummary {
    fn with_unknown_method_note(mut self, event_kind: McpProtocolEventKind, method: &str) -> Self {
        if event_kind == McpProtocolEventKind::Unknown {
            self.status = Some(format!(
                "unknown method {}",
                bound_protocol_event_text(&redact_protocol_event_text(method))
            ));
        }
        self
    }
}

fn extract_safe_string(params: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = nested_string_value(params, key) {
            return Some(redact_protocol_event_text(value));
        }
    }
    if let Some(data) = params.get("data").and_then(Value::as_str) {
        return Some(redact_protocol_event_text(data));
    }
    None
}

fn nested_string_value<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str).or_else(|| {
        value
            .get("data")
            .and_then(|data| data.get(key))
            .and_then(Value::as_str)
    })
}

fn nested_field_present(value: &Value, key: &str) -> bool {
    value.get(key).is_some() || value.get("data").and_then(|data| data.get(key)).is_some()
}

fn extract_number(params: &Value, key: &str) -> Option<f64> {
    params.get(key).and_then(Value::as_f64).or_else(|| {
        params
            .get("data")
            .and_then(|data| data.get(key))
            .and_then(Value::as_f64)
    })
}

fn rendered_value_len(value: &Value) -> usize {
    value.to_string().chars().count()
}

fn redact_protocol_event_text(value: &str) -> String {
    bound_protocol_event_text(&redact_secret_like_text(value))
}

fn bound_protocol_event_text(value: &str) -> String {
    let mut chars = value.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_PROTOCOL_EVENT_TEXT_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn value_contains_secret_like(value: &Value) -> bool {
    match value {
        Value::String(value) => is_secret_like(value),
        Value::Array(values) => values.iter().any(value_contains_secret_like),
        Value::Object(values) => values
            .iter()
            .any(|(key, value)| is_secret_like(key) || value_contains_secret_like(value)),
        _ => false,
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
        || normalized.contains("authorization")
        || normalized.contains("bearer")
        || normalized.contains("provider_payload")
        || normalized.contains("raw_prompt")
        || normalized.starts_with("sk_")
}
