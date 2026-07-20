use std::collections::BTreeMap;

use ruse_types::Timestamp;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{UiApprovalPreviewSummary, UiApprovalRequest, UiEventPayload, UiJsonFrame};

const PUBLIC_TEXT_LIMIT: usize = 160;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalQueueSnapshot {
    pub pending: Vec<UiApprovalQueueItem>,
    pub groups: Vec<UiApprovalQueueGroup>,
    pub pending_count: u32,
    pub expired_count: u32,
    pub diagnostic_count: u32,
    pub diagnostics: Vec<UiApprovalQueueDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalQueueItem {
    pub request_id: String,
    pub status: String,
    pub action: String,
    pub resource_kind: String,
    pub resource: String,
    pub reason: String,
    pub preview: UiApprovalPreviewSummary,
    pub max_duration_seconds: Option<u64>,
    pub expires_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalQueueGroup {
    pub resource_kind: String,
    pub action: String,
    pub status: String,
    pub count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiApprovalQueueDiagnostic {
    pub severity: UiApprovalQueueDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiApprovalQueueDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum UiApprovalQueueError {
    #[error("approval queue is invalid")]
    Invalid {
        diagnostics: Vec<UiApprovalQueueDiagnostic>,
    },
}

pub fn approval_queue_snapshot_from_requests(
    requests: Vec<UiApprovalRequest>,
) -> Result<UiApprovalQueueSnapshot, UiApprovalQueueError> {
    let mut by_id = BTreeMap::<String, UiApprovalRequest>::new();
    for request in requests {
        by_id.insert(request.request_id.clone(), request);
    }
    let mut pending = Vec::new();
    let mut expired_count = 0_u32;
    for request in by_id.into_values() {
        let status = request.status.to_ascii_lowercase();
        if status == "expired" {
            expired_count = expired_count.saturating_add(1);
            continue;
        }
        if status == "pending" {
            pending.push(UiApprovalQueueItem::from_request(&request));
        }
    }
    pending.sort_by(|left, right| {
        left.status
            .cmp(&right.status)
            .then_with(|| left.resource_kind.cmp(&right.resource_kind))
            .then_with(|| left.action.cmp(&right.action))
            .then_with(|| left.resource.cmp(&right.resource))
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    let groups = approval_queue_groups(&pending);
    let pending_count = pending.len().try_into().unwrap_or(u32::MAX);
    Ok(UiApprovalQueueSnapshot {
        pending,
        groups,
        pending_count,
        expired_count,
        diagnostic_count: 0,
        diagnostics: Vec::new(),
    })
}

pub fn approval_queue_snapshot_from_events(
    frames: &[UiJsonFrame],
) -> Result<UiApprovalQueueSnapshot, UiApprovalQueueError> {
    let mut by_id = BTreeMap::<String, UiApprovalRequest>::new();
    for frame in frames {
        let UiJsonFrame::Event(event) = frame else {
            continue;
        };
        match &event.payload {
            UiEventPayload::ApprovalRequested(request) => {
                by_id.insert(request.request_id.clone(), (**request).clone());
            }
            UiEventPayload::ApprovalResolved { request_id, .. } => {
                by_id.remove(request_id);
            }
            _ => {}
        }
    }
    approval_queue_snapshot_from_requests(by_id.into_values().collect())
}

impl UiApprovalQueueItem {
    #[must_use]
    pub fn from_request(request: &UiApprovalRequest) -> Self {
        Self {
            request_id: bounded_public_text(&request.request_id),
            status: bounded_public_text(&request.status),
            action: bounded_public_text(&request.action),
            resource_kind: bounded_public_text(&request.resource_kind),
            resource: bounded_public_text(&request.resource),
            reason: bounded_public_text(&request.reason),
            preview: UiApprovalPreviewSummary {
                kind: bounded_public_text(&request.preview.kind),
                resource: bounded_public_text(&request.preview.resource),
                summary: bounded_public_text(&request.preview.summary),
                content_hash: request
                    .preview
                    .content_hash
                    .as_ref()
                    .map(|value| bounded_public_text(value)),
                truncated: request.preview.truncated,
                redacted: request.preview.redacted,
            },
            max_duration_seconds: request.max_duration_seconds,
            expires_at: request.expires_at,
        }
    }
}

fn approval_queue_groups(pending: &[UiApprovalQueueItem]) -> Vec<UiApprovalQueueGroup> {
    let mut counts = BTreeMap::<(String, String, String), u32>::new();
    for item in pending {
        let key = (
            item.resource_kind.clone(),
            item.action.clone(),
            item.status.clone(),
        );
        *counts.entry(key).or_default() += 1;
    }
    counts
        .into_iter()
        .map(
            |((resource_kind, action, status), count)| UiApprovalQueueGroup {
                resource_kind,
                action,
                status,
                count,
            },
        )
        .collect()
}

fn bounded_public_text(value: &str) -> String {
    let redacted = redact_secret_like_text(value);
    let trimmed = redacted.trim();
    if trimmed.chars().count() > PUBLIC_TEXT_LIMIT {
        trimmed.chars().take(PUBLIC_TEXT_LIMIT).collect()
    } else {
        trimmed.to_string()
    }
}

fn redact_secret_like_text(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            if is_secret_like(word) {
                "[redacted]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_secret_like(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with("sk-")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
}
