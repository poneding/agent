use agent_types::{ActorRef, ApprovalRequestId, CheckpointId, Timestamp};
use serde::{Deserialize, Serialize};

use crate::{CapabilityAction, PermissionRequest, PolicyError, ResourceScope};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
    Cancelled,
    Consumed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalPreviewSummary {
    pub kind: String,
    pub resource: String,
    pub summary: String,
    pub content_hash: Option<String>,
    pub truncated: bool,
    pub redacted: bool,
}

impl ApprovalPreviewSummary {
    pub fn new(
        kind: impl Into<String>,
        resource: impl Into<String>,
        summary: impl Into<String>,
    ) -> Result<Self, PolicyError> {
        let kind = non_empty(kind, "approval preview kind")?;
        let resource = non_empty(resource, "approval preview resource")?;
        let summary = non_empty(summary, "approval preview summary")?;

        Ok(Self {
            kind,
            resource,
            summary,
            content_hash: None,
            truncated: false,
            redacted: false,
        })
    }

    #[must_use]
    pub fn with_content_hash(mut self, content_hash: impl Into<String>) -> Self {
        self.content_hash = Some(content_hash.into());
        self
    }

    #[must_use]
    pub const fn with_truncated(mut self, truncated: bool) -> Self {
        self.truncated = truncated;
        self
    }

    #[must_use]
    pub const fn with_redacted(mut self, redacted: bool) -> Self {
        self.redacted = redacted;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalLeaseScope {
    pub actor: ActorRef,
    pub action: CapabilityAction,
    pub resource: ResourceScope,
    pub max_duration_seconds: Option<u64>,
}

impl ApprovalLeaseScope {
    #[must_use]
    pub fn from_request(request: &PermissionRequest, max_duration_seconds: Option<u64>) -> Self {
        Self {
            actor: request.actor.clone(),
            action: request.action,
            resource: request.scope.clone(),
            max_duration_seconds,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalResumeToken {
    pub checkpoint_id: CheckpointId,
    pub label: String,
}

impl ApprovalResumeToken {
    pub fn new(checkpoint_id: CheckpointId, label: impl Into<String>) -> Result<Self, PolicyError> {
        Ok(Self {
            checkpoint_id,
            label: non_empty(label, "approval resume token label")?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub request_id: ApprovalRequestId,
    pub decision: ApprovalDecision,
    pub reason: String,
    pub lease_duration_seconds: Option<u64>,
    pub resolved_at: Timestamp,
}

impl ApprovalResponse {
    pub fn allow(
        request_id: ApprovalRequestId,
        reason: impl Into<String>,
        lease_duration_seconds: Option<u64>,
        resolved_at: Timestamp,
    ) -> Result<Self, PolicyError> {
        Ok(Self {
            request_id,
            decision: ApprovalDecision::Allow,
            reason: non_empty(reason, "approval response reason")?,
            lease_duration_seconds,
            resolved_at,
        })
    }

    pub fn deny(
        request_id: ApprovalRequestId,
        reason: impl Into<String>,
        resolved_at: Timestamp,
    ) -> Result<Self, PolicyError> {
        Ok(Self {
            request_id,
            decision: ApprovalDecision::Deny,
            reason: non_empty(reason, "approval response reason")?,
            lease_duration_seconds: None,
            resolved_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApprovalLeaseGrant {
    pub request_id: ApprovalRequestId,
    pub scope: ApprovalLeaseScope,
    pub duration_seconds: Option<u64>,
    pub issued_at: Timestamp,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: ApprovalRequestId,
    pub permission_request: PermissionRequest,
    pub preview: ApprovalPreviewSummary,
    pub lease_scope: ApprovalLeaseScope,
    pub resume_token: Option<ApprovalResumeToken>,
    pub status: ApprovalStatus,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub resolved_at: Option<Timestamp>,
    pub response: Option<ApprovalResponse>,
}

impl PendingApproval {
    pub fn new(
        permission_request: PermissionRequest,
        preview: ApprovalPreviewSummary,
        lease_scope: ApprovalLeaseScope,
        resume_token: Option<ApprovalResumeToken>,
        created_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, PolicyError> {
        Self::new_with_id(
            ApprovalRequestId::new(),
            permission_request,
            preview,
            lease_scope,
            resume_token,
            created_at,
            expires_at,
        )
    }

    pub fn new_with_id(
        id: ApprovalRequestId,
        permission_request: PermissionRequest,
        preview: ApprovalPreviewSummary,
        lease_scope: ApprovalLeaseScope,
        resume_token: Option<ApprovalResumeToken>,
        created_at: Timestamp,
        expires_at: Option<Timestamp>,
    ) -> Result<Self, PolicyError> {
        if expires_at.is_some_and(|expires_at| expires_at <= created_at) {
            return Err(invalid("approval expiration must be after creation"));
        }
        if lease_scope.actor != permission_request.actor
            || lease_scope.action != permission_request.action
            || lease_scope.resource != permission_request.scope
        {
            return Err(invalid(
                "approval lease scope must match permission request scope",
            ));
        }

        Ok(Self {
            id,
            permission_request,
            preview,
            lease_scope,
            resume_token,
            status: ApprovalStatus::Pending,
            created_at,
            expires_at,
            resolved_at: None,
            response: None,
        })
    }

    pub fn approve(
        &mut self,
        response: ApprovalResponse,
    ) -> Result<ApprovalLeaseGrant, PolicyError> {
        self.ensure_pending_response(&response, ApprovalDecision::Allow)?;
        let duration_seconds = clamp_duration(
            response.lease_duration_seconds,
            self.lease_scope.max_duration_seconds,
        );
        let issued_at = response.resolved_at;
        self.status = ApprovalStatus::Approved;
        self.resolved_at = Some(response.resolved_at);
        self.response = Some(response);

        Ok(ApprovalLeaseGrant {
            request_id: self.id,
            scope: self.lease_scope.clone(),
            duration_seconds,
            issued_at,
        })
    }

    pub fn deny(&mut self, response: ApprovalResponse) -> Result<(), PolicyError> {
        self.ensure_pending_response(&response, ApprovalDecision::Deny)?;
        self.status = ApprovalStatus::Denied;
        self.resolved_at = Some(response.resolved_at);
        self.response = Some(response);
        Ok(())
    }

    pub fn expire(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError> {
        self.ensure_status(ApprovalStatus::Pending, "expire")?;
        self.status = ApprovalStatus::Expired;
        self.resolved_at = Some(resolved_at);
        Ok(())
    }

    pub fn cancel(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError> {
        self.ensure_status(ApprovalStatus::Pending, "cancel")?;
        self.status = ApprovalStatus::Cancelled;
        self.resolved_at = Some(resolved_at);
        Ok(())
    }

    pub fn consume(&mut self, resolved_at: Timestamp) -> Result<(), PolicyError> {
        self.ensure_status(ApprovalStatus::Approved, "consume")?;
        self.status = ApprovalStatus::Consumed;
        self.resolved_at = Some(resolved_at);
        Ok(())
    }

    fn ensure_pending_response(
        &self,
        response: &ApprovalResponse,
        expected: ApprovalDecision,
    ) -> Result<(), PolicyError> {
        self.ensure_status(ApprovalStatus::Pending, "resolve")?;
        if response.request_id != self.id {
            return Err(invalid("approval response request id does not match"));
        }
        if response.decision != expected {
            return Err(invalid("approval response decision does not match"));
        }
        Ok(())
    }

    fn ensure_status(&self, expected: ApprovalStatus, action: &str) -> Result<(), PolicyError> {
        if self.status != expected {
            return Err(invalid(format!(
                "cannot {action} approval in {:?} state",
                self.status
            )));
        }
        Ok(())
    }
}

fn clamp_duration(requested: Option<u64>, maximum: Option<u64>) -> Option<u64> {
    match (requested, maximum) {
        (Some(requested), Some(maximum)) => Some(requested.min(maximum)),
        (Some(requested), None) => Some(requested),
        (None, Some(maximum)) => Some(maximum),
        (None, None) => None,
    }
}

fn non_empty(value: impl Into<String>, field: &str) -> Result<String, PolicyError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} cannot be empty")));
    }
    Ok(value)
}

fn invalid(message: impl Into<String>) -> PolicyError {
    PolicyError::InvalidRequest(message.into())
}
