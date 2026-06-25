use agent_policy::{CapabilityAction, ResourceScope};
use agent_types::Timestamp;
use serde::{Deserialize, Serialize};

use crate::DesktopError;

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
