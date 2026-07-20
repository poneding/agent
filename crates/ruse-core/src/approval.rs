use std::collections::BTreeMap;

use ruse_policy::{
    ApprovalDecision, ApprovalLeaseGrant, ApprovalResponse, ApprovalStatus, PendingApproval,
    PermissionLease, PermissionRequest, PolicyDecision, PolicyEngine, PolicyError,
};
use ruse_storage::{IdempotencyKey, IdempotencyStore};
use ruse_tool::ToolRegistry;
use ruse_types::{
    ActorRef, ApprovalConsumed, ApprovalRequestId, ApprovalResolved, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp,
};
use thiserror::Error;

use crate::idempotency::IdempotentToolExecutor;
use crate::{RuntimeIds, ToolCallRequest, ToolLoopOutcome};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuspendedToolApproval {
    pub ids: RuntimeIds,
    pub model_turn_index: u32,
    pub call: ToolCallRequest,
    pub approval: PendingApproval,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalRecoveryView {
    pub request_id: ApprovalRequestId,
    pub checkpoint_id: Option<ruse_types::CheckpointId>,
    pub action: String,
    pub resource_kind: String,
    pub resource: String,
    pub reason: String,
}

impl ApprovalRecoveryView {
    pub fn pending_from_events<'a>(
        events: impl IntoIterator<Item = &'a RuntimeEvent>,
    ) -> Vec<Self> {
        let mut pending = BTreeMap::new();

        for event in events {
            match &event.payload {
                RuntimeEventPayload::ApprovalRequested(requested) => {
                    pending.insert(
                        requested.request_id,
                        Self {
                            request_id: requested.request_id,
                            checkpoint_id: requested.checkpoint_id,
                            action: requested.action.clone(),
                            resource_kind: requested.resource_kind.clone(),
                            resource: requested.resource.clone(),
                            reason: requested.reason.clone(),
                        },
                    );
                }
                RuntimeEventPayload::ApprovalResolved(resolved) => {
                    pending.remove(&resolved.request_id);
                }
                RuntimeEventPayload::ApprovalExpired(expired) => {
                    pending.remove(&expired.request_id);
                }
                RuntimeEventPayload::ApprovalCancelled(cancelled) => {
                    pending.remove(&cancelled.request_id);
                }
                RuntimeEventPayload::ApprovalConsumed(consumed) => {
                    pending.remove(&consumed.request_id);
                }
                _ => {}
            }
        }

        pending.into_values().collect()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ApprovalResumeStatus {
    Executed,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalResumeOutcome {
    pub status: ApprovalResumeStatus,
    pub tool: Option<ToolLoopOutcome>,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ApprovalResumeError {
    #[error("approval request is stale: {0}")]
    StaleRequest(ApprovalRequestId),
    #[error("approval request expired: {0}")]
    Expired(ApprovalRequestId),
    #[error("approval policy failed: {0}")]
    Policy(String),
    #[error("approved tool execution failed: {0}")]
    Tool(String),
}

#[derive(Default)]
pub struct ApprovalCoordinator {
    pending: BTreeMap<ApprovalRequestId, SuspendedToolApproval>,
}

impl ApprovalCoordinator {
    pub fn register_suspended_tool(
        &mut self,
        ids: RuntimeIds,
        model_turn_index: u32,
        call: ToolCallRequest,
        approval: PendingApproval,
    ) -> Result<(), ApprovalResumeError> {
        if approval.status != ApprovalStatus::Pending {
            return Err(ApprovalResumeError::Policy(
                "only pending approvals can be registered".to_string(),
            ));
        }

        self.pending.insert(
            approval.id,
            SuspendedToolApproval {
                ids,
                model_turn_index,
                call,
                approval,
            },
        );
        Ok(())
    }

    pub fn resume_tool_approval<S>(
        &mut self,
        response: ApprovalResponse,
        registry: &ToolRegistry,
        effects: &mut S,
    ) -> Result<ApprovalResumeOutcome, ApprovalResumeError>
    where
        S: IdempotencyStore + ?Sized,
    {
        let Some(mut suspended) = self.pending.remove(&response.request_id) else {
            return Err(ApprovalResumeError::StaleRequest(response.request_id));
        };

        if suspended
            .approval
            .expires_at
            .is_some_and(|expires_at| expires_at <= response.resolved_at)
        {
            suspended
                .approval
                .expire(response.resolved_at)
                .map_err(policy_error)?;
            return Err(ApprovalResumeError::Expired(response.request_id));
        }

        match response.decision {
            ApprovalDecision::Deny => {
                suspended.approval.deny(response).map_err(policy_error)?;
                let events = vec![event(
                    suspended.ids,
                    RuntimeEventPayload::ApprovalResolved(approval_resolved_event(
                        &suspended.approval,
                    )),
                )];
                Ok(ApprovalResumeOutcome {
                    status: ApprovalResumeStatus::Denied,
                    tool: None,
                    events,
                })
            }
            ApprovalDecision::Allow => {
                let grant = suspended.approval.approve(response).map_err(policy_error)?;
                let mut events = vec![event(
                    suspended.ids,
                    RuntimeEventPayload::ApprovalResolved(approval_resolved_event(
                        &suspended.approval,
                    )),
                )];
                let key = approval_idempotency_key(
                    suspended.ids,
                    suspended.model_turn_index,
                    &suspended.call,
                )?;
                let policy = ApprovalGrantPolicy { grant };
                let tool_outcome = IdempotentToolExecutor::default()
                    .run(
                        suspended.ids,
                        key,
                        suspended.call.clone(),
                        registry,
                        &policy,
                        effects,
                    )
                    .map_err(|error| ApprovalResumeError::Tool(error.to_string()))?
                    .tool;
                events.extend(tool_outcome.events.clone());
                suspended
                    .approval
                    .consume(Timestamp::now_utc())
                    .map_err(policy_error)?;
                events.push(event(
                    suspended.ids,
                    RuntimeEventPayload::ApprovalConsumed(ApprovalConsumed {
                        request_id: suspended.approval.id,
                        call_id: suspended.call.call_id,
                        tool_name: suspended.call.tool_name,
                    }),
                ));

                Ok(ApprovalResumeOutcome {
                    status: ApprovalResumeStatus::Executed,
                    tool: Some(tool_outcome),
                    events,
                })
            }
        }
    }
}

struct ApprovalGrantPolicy {
    grant: ApprovalLeaseGrant,
}

impl PolicyEngine for ApprovalGrantPolicy {
    fn decide(&self, request: &PermissionRequest) -> Result<PolicyDecision, PolicyError> {
        if request.actor == self.grant.scope.actor
            && request.action == self.grant.scope.action
            && request.scope == self.grant.scope.resource
        {
            Ok(PolicyDecision::Allow {
                reason: "approved by pending approval response".to_string(),
            })
        } else {
            Ok(PolicyDecision::Deny {
                reason: "approval grant does not cover requested capability".to_string(),
            })
        }
    }

    fn issue_lease(
        &self,
        request: &PermissionRequest,
        decision: PolicyDecision,
    ) -> Result<Option<PermissionLease>, PolicyError> {
        if matches!(decision, PolicyDecision::Allow { .. }) {
            return Ok(Some(PermissionLease::new(request.id.clone(), decision)));
        }

        Ok(None)
    }
}

fn approval_resolved_event(approval: &PendingApproval) -> ApprovalResolved {
    let response = approval.response.as_ref();
    ApprovalResolved {
        request_id: approval.id,
        status: approval_status_name(approval.status).to_string(),
        decision: response.map(|response| approval_decision_name(response.decision).to_string()),
        reason: response.map(|response| response.reason.clone()),
        lease_duration_seconds: response.and_then(|response| response.lease_duration_seconds),
    }
}

fn approval_idempotency_key(
    ids: RuntimeIds,
    model_turn_index: u32,
    call: &ToolCallRequest,
) -> Result<IdempotencyKey, ApprovalResumeError> {
    let input_hash = stable_tool_input_hash(call);
    IdempotencyKey::new(format!(
        "turn:{}:{}:{}:{}:{}:{}:{}",
        ids.workspace_id,
        ids.conversation_id,
        ids.turn_id,
        model_turn_index,
        call.call_id,
        call.tool_name,
        input_hash
    ))
    .map_err(|error| ApprovalResumeError::Policy(error.to_string()))
}

fn stable_tool_input_hash(call: &ToolCallRequest) -> String {
    let input =
        serde_json::to_string(&call.input_json).unwrap_or_else(|_| call.input_json.to_string());
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv1a(hash, call.tool_name.as_bytes());
    hash = fnv1a(hash, b"\0");
    hash = fnv1a(hash, input.as_bytes());
    format!("fnv1a64:{hash:016x}")
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn event(ids: RuntimeIds, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: ids.workspace_id,
        conversation_id: ids.conversation_id,
        turn_id: Some(ids.turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: ids.correlation_id,
        causation_id: None,
        payload,
    }
}

fn policy_error(error: PolicyError) -> ApprovalResumeError {
    ApprovalResumeError::Policy(error.to_string())
}

fn approval_status_name(status: ApprovalStatus) -> &'static str {
    match status {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Denied => "denied",
        ApprovalStatus::Expired => "expired",
        ApprovalStatus::Cancelled => "cancelled",
        ApprovalStatus::Consumed => "consumed",
    }
}

fn approval_decision_name(decision: ApprovalDecision) -> &'static str {
    match decision {
        ApprovalDecision::Allow => "allow",
        ApprovalDecision::Deny => "deny",
    }
}
