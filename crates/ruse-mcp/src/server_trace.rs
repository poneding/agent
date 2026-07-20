use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_MAX_TRACE_ENTRIES: usize = 128;
const MAX_TRACE_FIELD_CHARS: usize = 128;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerTraceEntry {
    pub method: String,
    pub outcome: String,
    pub initialized: bool,
}

impl McpServerTraceEntry {
    #[must_use]
    pub fn new(method: impl Into<String>, outcome: impl Into<String>, initialized: bool) -> Self {
        Self {
            method: redact_trace_text(method.into()),
            outcome: redact_trace_text(outcome.into()),
            initialized,
        }
    }

    #[must_use]
    pub fn from_frame_response(frame: &str, response: &str, initialized: bool) -> Self {
        Self::new(
            method_from_frame(frame),
            outcome_from_response(response),
            initialized,
        )
    }

    #[must_use]
    pub fn rejected_frame(frame: &str, outcome: impl Into<String>, initialized: bool) -> Self {
        Self::new(method_from_frame(frame), outcome, initialized)
    }

    #[must_use]
    pub fn oversized_frame(outcome: impl Into<String>, initialized: bool) -> Self {
        Self::new("oversized", outcome, initialized)
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        !(self.outcome == "ok" || self.outcome == "notification")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerTraceSnapshot {
    pub total_entries: u64,
    pub error_entries: u64,
    pub truncated: bool,
    pub entries: Vec<McpServerTraceEntry>,
}

impl McpServerTraceSnapshot {
    #[must_use]
    pub fn new(
        total_entries: u64,
        error_entries: u64,
        truncated: bool,
        entries: Vec<McpServerTraceEntry>,
    ) -> Self {
        Self {
            total_entries,
            error_entries,
            truncated,
            entries,
        }
    }

    #[must_use]
    pub fn from_entries(entries: &[McpServerTraceEntry], max_entries: usize) -> Self {
        let total_entries = entries.len() as u64;
        let error_entries = entries.iter().filter(|entry| entry.is_error()).count() as u64;
        let max_entries = if max_entries == 0 {
            DEFAULT_MAX_TRACE_ENTRIES
        } else {
            max_entries
        };
        let truncated = entries.len() > max_entries;
        let start = entries.len().saturating_sub(max_entries);
        Self::new(
            total_entries,
            error_entries,
            truncated,
            entries[start..].to_vec(),
        )
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new(0, 0, false, Vec::new())
    }
}

#[must_use]
pub fn format_mcp_server_trace_summary(snapshot: &McpServerTraceSnapshot) -> String {
    let mut output = format!(
        "mcp server trace total={} errors={} truncated={}\n",
        snapshot.total_entries, snapshot.error_entries, snapshot.truncated
    );
    for entry in &snapshot.entries {
        output.push_str(&format!(
            "- method={} outcome={} initialized={}\n",
            entry.method, entry.outcome, entry.initialized
        ));
    }
    output
}

fn method_from_frame(frame: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(frame.trim_end()) else {
        return "malformed".to_string();
    };
    value
        .get("method")
        .and_then(Value::as_str)
        .filter(|method| !method.trim().is_empty())
        .map_or_else(|| "unknown".to_string(), redact_trace_text)
}

fn outcome_from_response(response: &str) -> String {
    let trimmed = response.trim_end();
    if trimmed.is_empty() {
        return "notification".to_string();
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return "error:unparseable_response".to_string();
    };
    let Some(error) = value.get("error") else {
        return "ok".to_string();
    };
    let message = error.get("message").and_then(Value::as_str).unwrap_or("");
    if message.to_ascii_lowercase().contains("policy denied") {
        return "denied".to_string();
    }
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
    match code {
        -32700 => "error:parse",
        -32600 => "error:invalid_request",
        -32601 => "error:method_not_found",
        -32602 => "error:invalid_params",
        -32603 => "error:internal",
        _ => "error",
    }
    .to_string()
}

fn redact_trace_text(value: impl AsRef<str>) -> String {
    let value = value.as_ref().trim();
    if value.is_empty() {
        return "unknown".to_string();
    }
    if is_secret_like(value) {
        return REDACTED.to_string();
    }
    bound_chars(value, MAX_TRACE_FIELD_CHARS)
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
