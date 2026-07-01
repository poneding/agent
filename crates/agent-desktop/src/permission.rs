use agent_policy::{CapabilityAction, ResourceScope};
use agent_types::Timestamp;
use agent_ui_bridge::UiApprovalRequest;
use serde::{Deserialize, Serialize};

use crate::DesktopError;

const MAX_PERMISSION_TEXT_CHARS: usize = 240;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDialogStatus {
    Pending,
    Allowed,
    Denied,
    Expired,
}

impl PermissionDialogStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Allowed | Self::Denied | Self::Expired)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopPermissionDecision {
    AllowOnce,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissionDialog {
    pub request_id: String,
    pub action: CapabilityAction,
    pub scope: ResourceScope,
    pub reason: String,
    pub status: PermissionDialogStatus,
    pub created_at: Timestamp,
    pub decided_at: Option<Timestamp>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopApprovalReview {
    pub request_id: String,
    pub status: DesktopApprovalReviewStatus,
    pub action_label: String,
    pub resource_kind: String,
    pub resource_label: String,
    pub policy_rationale: String,
    pub preview: DesktopApprovalPreview,
    pub lease_duration_seconds: Option<u64>,
    pub expires_at: Option<Timestamp>,
    pub decision_enabled: bool,
    pub disabled_reason: Option<String>,
}

impl DesktopApprovalReview {
    pub fn from_ui_request(request: &UiApprovalRequest) -> Result<Self, DesktopError> {
        let status = DesktopApprovalReviewStatus::from_status_label(&request.status);
        let disabled_reason = disabled_reason_for_status(status);
        Ok(Self {
            request_id: validate_bounded("approval request id", request.request_id.clone())?.0,
            status,
            action_label: validate_bounded("approval action", request.action.clone())?.0,
            resource_kind: validate_bounded(
                "approval resource kind",
                request.resource_kind.clone(),
            )?
            .0,
            resource_label: validate_bounded("approval resource", request.resource.clone())?.0,
            policy_rationale: validate_bounded("approval rationale", request.reason.clone())?.0,
            preview: DesktopApprovalPreview::from_ui_preview(&request.preview)?,
            lease_duration_seconds: request.max_duration_seconds,
            expires_at: request.expires_at,
            decision_enabled: disabled_reason.is_none(),
            disabled_reason,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopApprovalReviewStatus {
    Pending,
    Allowed,
    Denied,
    Expired,
    Resolved,
}

impl DesktopApprovalReviewStatus {
    #[must_use]
    pub fn from_status_label(status: &str) -> Self {
        match status.trim().to_ascii_lowercase().as_str() {
            "allowed" | "allow" | "approved" => Self::Allowed,
            "denied" | "deny" | "rejected" => Self::Denied,
            "expired" => Self::Expired,
            "resolved" | "consumed" => Self::Resolved,
            _ => Self::Pending,
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DesktopApprovalPreview {
    pub kind: String,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

impl DesktopApprovalPreview {
    pub fn from_ui_preview(
        preview: &agent_ui_bridge::UiApprovalPreviewSummary,
    ) -> Result<Self, DesktopError> {
        let (kind, kind_truncated) =
            validate_bounded("approval preview kind", preview.kind.clone())?;
        let (resource, resource_truncated) =
            validate_bounded("approval preview resource", preview.resource.clone())?;
        let (summary, summary_truncated) =
            validate_bounded("approval preview summary", preview.summary.clone())?;
        Ok(Self {
            kind,
            resource,
            summary,
            content_hash: preview.content_hash.clone(),
            truncated: preview.truncated
                || kind_truncated
                || resource_truncated
                || summary_truncated,
            redacted: preview.redacted,
        })
    }
}

impl PermissionDialog {
    pub fn new(
        request_id: impl Into<String>,
        action: CapabilityAction,
        scope: ResourceScope,
        reason: impl Into<String>,
        created_at: Timestamp,
    ) -> Result<Self, DesktopError> {
        Ok(Self {
            request_id: validate_non_empty("permission request id", request_id.into())?,
            action,
            scope,
            reason: validate_non_empty("permission reason", reason.into())?,
            status: PermissionDialogStatus::Pending,
            created_at,
            decided_at: None,
        })
    }
}

pub fn resolve_permission_dialog(
    dialog: &PermissionDialog,
    decision: DesktopPermissionDecision,
    now: Timestamp,
) -> Result<PermissionDialog, DesktopError> {
    transition_permission_dialog(
        dialog,
        match decision {
            DesktopPermissionDecision::AllowOnce => PermissionDialogStatus::Allowed,
            DesktopPermissionDecision::Deny => PermissionDialogStatus::Denied,
        },
        now,
    )
}

pub fn expire_permission_dialog(
    dialog: &PermissionDialog,
    now: Timestamp,
) -> Result<PermissionDialog, DesktopError> {
    transition_permission_dialog(dialog, PermissionDialogStatus::Expired, now)
}

fn transition_permission_dialog(
    dialog: &PermissionDialog,
    status: PermissionDialogStatus,
    now: Timestamp,
) -> Result<PermissionDialog, DesktopError> {
    if dialog.status.is_terminal() {
        return Err(DesktopError::InvalidPermission(format!(
            "terminal permission dialog {:?} cannot transition",
            dialog.status
        )));
    }
    if status == PermissionDialogStatus::Pending {
        return Err(DesktopError::InvalidPermission(
            "permission dialog cannot transition back to pending".to_string(),
        ));
    }

    let mut next = dialog.clone();
    next.status = status;
    next.decided_at = Some(now);
    Ok(next)
}

pub(crate) fn validate_non_empty(label: &str, value: String) -> Result<String, DesktopError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(DesktopError::InvalidPermission(format!(
            "{label} cannot be empty"
        )));
    }
    Ok(value)
}

fn disabled_reason_for_status(status: DesktopApprovalReviewStatus) -> Option<String> {
    match status {
        DesktopApprovalReviewStatus::Pending => None,
        DesktopApprovalReviewStatus::Allowed => Some("approval already allowed".to_string()),
        DesktopApprovalReviewStatus::Denied => Some("approval already denied".to_string()),
        DesktopApprovalReviewStatus::Expired => Some("approval expired".to_string()),
        DesktopApprovalReviewStatus::Resolved => Some("approval already resolved".to_string()),
    }
}

fn validate_bounded(label: &str, value: String) -> Result<(String, bool), DesktopError> {
    let value = validate_non_empty(label, value)?;
    Ok(bound_permission_text(&value))
}

fn bound_permission_text(value: &str) -> (String, bool) {
    let redacted = redact_secret_like_text(value);
    if redacted.chars().count() > MAX_PERMISSION_TEXT_CHARS {
        (
            redacted.chars().take(MAX_PERMISSION_TEXT_CHARS).collect(),
            true,
        )
    } else {
        (redacted, false)
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
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.starts_with("sk-")
}
