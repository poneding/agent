use std::collections::BTreeSet;

use ruse_core::TraceEntry;
use ruse_types::ConversationId;
use serde::{Deserialize, Serialize};

use crate::DesktopError;

const MAX_LOG_MESSAGE_CHARS: usize = 240;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceTimelineSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceTimelineItem {
    pub seq: u64,
    pub kind: String,
    pub title: String,
    pub summary: String,
    pub severity: TraceTimelineSeverity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TraceTimeline {
    pub conversation_id: ConversationId,
    pub items: Vec<TraceTimelineItem>,
    pub logs: Vec<DesktopRuntimeLogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopRuntimeLogEntry {
    pub sequence: u64,
    pub severity: DesktopRuntimeLogSeverity,
    pub source: String,
    pub message: String,
    pub truncated: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopRuntimeLogSeverity {
    Info,
    Warning,
    Error,
}

pub fn build_trace_timeline(
    conversation_id: ConversationId,
    entries: Vec<TraceEntry>,
) -> Result<TraceTimeline, DesktopError> {
    let mut seen = BTreeSet::new();
    let mut items = Vec::with_capacity(entries.len());
    let mut logs = Vec::with_capacity(entries.len());

    for entry in entries {
        if !seen.insert(entry.seq) {
            return Err(DesktopError::InvalidTrace(format!(
                "duplicate trace sequence: {}",
                entry.seq
            )));
        }

        let kind = validate_non_empty("trace kind", entry.kind)?;
        let summary = sanitize_summary(validate_non_empty("trace summary", entry.summary)?);
        let severity = severity_from_kind(&kind);
        let (message, truncated) = bound_log_message(&summary);
        logs.push(DesktopRuntimeLogEntry {
            sequence: entry.seq,
            severity: log_severity(severity),
            source: kind.clone(),
            message,
            truncated,
        });
        items.push(TraceTimelineItem {
            seq: entry.seq,
            title: title_from_kind(&kind),
            severity,
            kind,
            summary,
        });
    }

    items.sort_by_key(|item| item.seq);
    logs.sort_by_key(|log| log.sequence);
    Ok(TraceTimeline {
        conversation_id,
        items,
        logs,
    })
}

fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidTrace(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn title_from_kind(kind: &str) -> String {
    kind.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut title = first.to_ascii_uppercase().to_string();
                    title.push_str(chars.as_str());
                    title
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn severity_from_kind(kind: &str) -> TraceTimelineSeverity {
    let kind = kind.to_ascii_lowercase();
    if kind.contains("error") || kind.contains("failed") || kind.contains("failure") {
        TraceTimelineSeverity::Error
    } else if kind.contains("warning") {
        TraceTimelineSeverity::Warning
    } else {
        TraceTimelineSeverity::Info
    }
}

fn sanitize_summary(summary: String) -> String {
    summary
        .split_whitespace()
        .map(|token| {
            if contains_secret_key(token) {
                "[redacted]".to_string()
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn bound_log_message(value: &str) -> (String, bool) {
    if value.chars().count() > MAX_LOG_MESSAGE_CHARS {
        (value.chars().take(MAX_LOG_MESSAGE_CHARS).collect(), true)
    } else {
        (value.to_string(), false)
    }
}

fn log_severity(severity: TraceTimelineSeverity) -> DesktopRuntimeLogSeverity {
    match severity {
        TraceTimelineSeverity::Info => DesktopRuntimeLogSeverity::Info,
        TraceTimelineSeverity::Warning => DesktopRuntimeLogSeverity::Warning,
        TraceTimelineSeverity::Error => DesktopRuntimeLogSeverity::Error,
    }
}

fn contains_secret_key(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
}
