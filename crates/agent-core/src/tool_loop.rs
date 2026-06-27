use std::collections::BTreeSet;

use agent_policy::{
    ApprovalLeaseScope, ApprovalPreviewSummary, ApprovalResumeToken, PendingApproval,
    PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
};
use agent_tool::{ToolInput, ToolName, ToolRegistry};
use agent_types::{
    ActorRef, ApprovalPreviewRecorded, ApprovalRequested, CheckpointId, CheckpointRecorded,
    PermissionDecisionRecorded, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    ToolCallProposed, ToolExecutionCompleted, ToolExecutionFailed, ToolExecutionStarted,
    ToolObservationRecorded, TurnState,
};
use thiserror::Error;

use crate::runtime::RuntimeIds;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCallRequest {
    pub call_id: String,
    pub tool_name: String,
    pub input_json: serde_json::Value,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ToolObservationMessage {
    pub call_id: String,
    pub tool_name: String,
    pub observation: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccumulatedToolCall {
    Ready(ToolCallRequest),
    Observation(ToolObservationMessage),
}

#[derive(Clone, Debug, Default)]
pub struct ToolCallAccumulator {
    seen_call_ids: BTreeSet<String>,
}

impl ToolCallAccumulator {
    #[must_use]
    pub fn push_tool_call(
        &mut self,
        call_id: impl Into<String>,
        tool_name: impl Into<String>,
        input_json: serde_json::Value,
    ) -> AccumulatedToolCall {
        let call_id = call_id.into();
        let tool_name = tool_name.into();

        if self.seen_call_ids.contains(&call_id) {
            return AccumulatedToolCall::Observation(observation_message(
                call_id,
                tool_name,
                "duplicate tool call id",
            ));
        }

        if let Err(error) = ToolName::new(tool_name.clone()) {
            return AccumulatedToolCall::Observation(observation_message(
                call_id,
                tool_name,
                format!("invalid tool name: {error}"),
            ));
        }

        if !input_json.is_object() {
            return AccumulatedToolCall::Observation(observation_message(
                call_id,
                tool_name,
                "tool input must be a JSON object",
            ));
        }

        self.seen_call_ids.insert(call_id.clone());
        AccumulatedToolCall::Ready(ToolCallRequest {
            call_id,
            tool_name,
            input_json,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ToolLoopStatus {
    Completed,
    Denied,
    WaitingApproval,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolLoopOutcome {
    pub status: ToolLoopStatus,
    pub turn_state: TurnState,
    pub observation: Option<String>,
    pub pending_approval: Option<PendingApproval>,
    pub events: Vec<RuntimeEvent>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ToolLoopError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("invalid tool name: {0}")]
    InvalidToolName(String),
    #[error("policy failed: {0}")]
    Policy(String),
}

#[derive(Clone, Debug, Default)]
pub struct ToolLoop;

impl ToolLoop {
    pub fn run<P>(
        &self,
        ids: RuntimeIds,
        call: ToolCallRequest,
        registry: &ToolRegistry,
        policy: &P,
    ) -> Result<ToolLoopOutcome, ToolLoopError>
    where
        P: PolicyEngine + ?Sized,
    {
        let mut events = vec![event(
            ids,
            RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
                call_id: call.call_id.clone(),
                tool_name: call.tool_name.clone(),
                input_json: call.input_json.clone(),
            }),
        )];

        let tool_name = ToolName::new(call.tool_name.clone())
            .map_err(|error| ToolLoopError::InvalidToolName(error.to_string()))?;
        let tool = registry
            .get(&tool_name)
            .ok_or_else(|| ToolLoopError::UnknownTool(call.tool_name.clone()))?;
        let request = PermissionRequest::new(
            ActorRef::system(),
            agent_policy::CapabilityAction::Read,
            ResourceScope::new(ResourceKind::Tool, call.tool_name.clone())
                .map_err(|error| ToolLoopError::Policy(error.to_string()))?,
            "tool execution requested",
            ids.correlation_id,
        )
        .map_err(|error| ToolLoopError::Policy(error.to_string()))?;
        let decision = policy
            .decide(&request)
            .map_err(|error| ToolLoopError::Policy(error.to_string()))?;

        events.push(event(
            ids,
            RuntimeEventPayload::PermissionDecisionRecorded(PermissionDecisionRecorded {
                request_id: request.id.clone(),
                decision: decision_name(&decision).to_string(),
                reason: decision_reason(&decision).to_string(),
            }),
        ));

        match decision {
            PolicyDecision::Allow { .. } => {
                events.push(event(
                    ids,
                    RuntimeEventPayload::ToolExecutionStarted(ToolExecutionStarted {
                        call_id: call.call_id.clone(),
                        tool_name: call.tool_name.clone(),
                    }),
                ));

                match tool.execute(ToolInput {
                    value: call.input_json,
                }) {
                    Ok(output) => {
                        let observation = observation_from_output(&output.value);
                        events.push(event(
                            ids,
                            RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
                                call_id: call.call_id.clone(),
                                output_json: output.value,
                            }),
                        ));
                        events.push(event(
                            ids,
                            RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
                                call_id: call.call_id,
                                observation: observation.clone(),
                            }),
                        ));
                        Ok(ToolLoopOutcome {
                            status: ToolLoopStatus::Completed,
                            turn_state: TurnState::Observed,
                            observation: Some(observation),
                            pending_approval: None,
                            events,
                        })
                    }
                    Err(error) => {
                        events.push(event(
                            ids,
                            RuntimeEventPayload::ToolExecutionFailed(ToolExecutionFailed {
                                call_id: call.call_id,
                                error_code: "tool_execution_failed".to_string(),
                                message: error.to_string(),
                                recoverable: true,
                            }),
                        ));
                        Ok(ToolLoopOutcome {
                            status: ToolLoopStatus::Failed,
                            turn_state: TurnState::Failed,
                            observation: None,
                            pending_approval: None,
                            events,
                        })
                    }
                }
            }
            PolicyDecision::Deny { .. } => Ok(ToolLoopOutcome {
                status: ToolLoopStatus::Denied,
                turn_state: TurnState::Failed,
                observation: None,
                pending_approval: None,
                events,
            }),
            PolicyDecision::Ask { .. } => {
                let checkpoint_id = CheckpointId::new();
                let pending_approval = pending_approval_for_tool(&request, &call, checkpoint_id)?;

                events.push(event(
                    ids,
                    RuntimeEventPayload::CheckpointRecorded(CheckpointRecorded {
                        checkpoint_id,
                        label: "before approved tool execution".to_string(),
                    }),
                ));
                events.push(event(
                    ids,
                    RuntimeEventPayload::ApprovalRequested(approval_requested_event(
                        &pending_approval,
                    )),
                ));

                Ok(ToolLoopOutcome {
                    status: ToolLoopStatus::WaitingApproval,
                    turn_state: TurnState::WaitingApproval,
                    observation: None,
                    pending_approval: Some(pending_approval),
                    events,
                })
            }
        }
    }
}

fn pending_approval_for_tool(
    request: &PermissionRequest,
    call: &ToolCallRequest,
    checkpoint_id: CheckpointId,
) -> Result<PendingApproval, ToolLoopError> {
    let preview = ApprovalPreviewSummary::new(
        "tool",
        call.tool_name.clone(),
        format!("Approve tool execution for {}", call.tool_name),
    )
    .map_err(|error| ToolLoopError::Policy(error.to_string()))?;
    let resume_token = ApprovalResumeToken::new(checkpoint_id, "before approved tool execution")
        .map_err(|error| ToolLoopError::Policy(error.to_string()))?;

    PendingApproval::new(
        request.clone(),
        preview,
        ApprovalLeaseScope::from_request(request, None),
        Some(resume_token),
        Timestamp::now_utc(),
        None,
    )
    .map_err(|error| ToolLoopError::Policy(error.to_string()))
}

fn approval_requested_event(approval: &PendingApproval) -> ApprovalRequested {
    ApprovalRequested {
        request_id: approval.id,
        status: "pending".to_string(),
        action: capability_action_name(approval.lease_scope.action).to_string(),
        resource_kind: resource_kind_name(approval.lease_scope.resource.kind).to_string(),
        resource: approval.lease_scope.resource.pattern.clone(),
        reason: approval.permission_request.reason.clone(),
        preview: ApprovalPreviewRecorded {
            kind: approval.preview.kind.clone(),
            resource: approval.preview.resource.clone(),
            summary: approval.preview.summary.clone(),
            content_hash: approval.preview.content_hash.clone(),
            truncated: approval.preview.truncated,
            redacted: approval.preview.redacted,
        },
        checkpoint_id: approval
            .resume_token
            .as_ref()
            .map(|token| token.checkpoint_id),
        max_duration_seconds: approval.lease_scope.max_duration_seconds,
        expires_at: approval.expires_at,
    }
}

fn event(ids: RuntimeIds, payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
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

fn decision_name(decision: &PolicyDecision) -> &'static str {
    match decision {
        PolicyDecision::Allow { .. } => "allow",
        PolicyDecision::Deny { .. } => "deny",
        PolicyDecision::Ask { .. } => "ask",
    }
}

fn decision_reason(decision: &PolicyDecision) -> &str {
    match decision {
        PolicyDecision::Allow { reason }
        | PolicyDecision::Deny { reason }
        | PolicyDecision::Ask { reason } => reason,
    }
}

fn observation_from_output(output: &serde_json::Value) -> String {
    output
        .get("content")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| output.to_string(), ToString::to_string)
}

fn observation_message(
    call_id: String,
    tool_name: String,
    observation: impl Into<String>,
) -> ToolObservationMessage {
    ToolObservationMessage {
        call_id,
        tool_name,
        observation: observation.into(),
        truncated: false,
    }
}

fn capability_action_name(action: agent_policy::CapabilityAction) -> &'static str {
    match action {
        agent_policy::CapabilityAction::Read => "read",
        agent_policy::CapabilityAction::Search => "search",
        agent_policy::CapabilityAction::Execute => "execute",
        agent_policy::CapabilityAction::Network => "network",
    }
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::File => "file",
        ResourceKind::Workspace => "workspace",
        ResourceKind::Command => "command",
        ResourceKind::Network => "network",
        ResourceKind::Tool => "tool",
    }
}
