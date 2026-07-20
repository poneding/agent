use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use agent_policy::{
    ApprovalLeaseScope, ApprovalPreviewSummary, ApprovalResumeToken, PendingApproval,
    PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope,
};
use agent_tool::{Tool, ToolInput, ToolName, ToolOutput, ToolRegistry};
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
    #[error("tool execution worker failed: {0}")]
    Worker(String),
}

#[derive(Clone, Debug, Default)]
pub struct ToolLoop;

impl ToolLoop {
    /// Synchronous tool loop for fixtures and non-async callers.
    /// Prefer [`Self::run_async`] on the interactive agent path so blocking
    /// tools (shell, filesystem) do not stall the Tokio runtime.
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
        let prepared = prepare_tool_call(ids, call, registry, policy)?;
        match prepared {
            PreparedToolCall::Terminal(outcome) => Ok(outcome),
            PreparedToolCall::Execute {
                mut events,
                call,
                tool,
                decision,
                request,
            } => {
                let started_at = Instant::now();
                events.push(execution_started_event(
                    ids,
                    &call,
                    &decision,
                    &request,
                ));
                let input_json = call.input_json.clone();
                Ok(finish_tool_execution(
                    ids,
                    call,
                    events,
                    started_at,
                    execute_tool_blocking(tool, input_json),
                ))
            }
        }
    }

    /// Async tool loop: policy stays on the async path; tool `execute` runs on
    /// Tokio's blocking pool so long shell/file work cannot stall streaming.
    pub async fn run_async<P>(
        &self,
        ids: RuntimeIds,
        call: ToolCallRequest,
        registry: &ToolRegistry,
        policy: &P,
    ) -> Result<ToolLoopOutcome, ToolLoopError>
    where
        P: PolicyEngine + ?Sized,
    {
        let prepared = prepare_tool_call(ids, call, registry, policy)?;
        match prepared {
            PreparedToolCall::Terminal(outcome) => Ok(outcome),
            PreparedToolCall::Execute {
                mut events,
                call,
                tool,
                decision,
                request,
            } => {
                let started_at = Instant::now();
                events.push(execution_started_event(
                    ids,
                    &call,
                    &decision,
                    &request,
                ));
                let input_json = call.input_json.clone();
                let execution = tokio::task::spawn_blocking(move || {
                    tool.execute(ToolInput { value: input_json })
                })
                .await
                .map_err(|error| ToolLoopError::Worker(error.to_string()))?;
                Ok(finish_tool_execution(
                    ids, call, events, started_at, execution,
                ))
            }
        }
    }
}

/// Run tool.execute off the async worker when a multi-thread Tokio runtime is
/// active (interactive TUI). Falls back to in-place execution on current-thread
/// runtimes and outside Tokio (unit tests) so we never call `block_in_place`
/// where it would panic.
fn execute_tool_blocking(
    tool: Arc<dyn Tool>,
    input_json: serde_json::Value,
) -> Result<ToolOutput, agent_tool::ToolError> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle)
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread =>
        {
            // block_in_place lets other async tasks progress on multi-thread
            // runtimes while we wait on the blocking pool for shell/fs work.
            tokio::task::block_in_place(|| {
                handle.block_on(async move {
                    tokio::task::spawn_blocking(move || {
                        tool.execute(ToolInput { value: input_json })
                    })
                    .await
                    .unwrap_or_else(|error| {
                        Err(agent_tool::ToolError::Execution(format!(
                            "tool worker join failed: {error}"
                        )))
                    })
                })
            })
        }
        _ => tool.execute(ToolInput { value: input_json }),
    }
}

enum PreparedToolCall {
    Terminal(ToolLoopOutcome),
    Execute {
        events: Vec<RuntimeEvent>,
        call: ToolCallRequest,
        tool: Arc<dyn Tool>,
        decision: PolicyDecision,
        request: PermissionRequest,
    },
}

fn prepare_tool_call<P>(
    ids: RuntimeIds,
    call: ToolCallRequest,
    registry: &ToolRegistry,
    policy: &P,
) -> Result<PreparedToolCall, ToolLoopError>
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
    let action = capability_action_for_tool(&call.tool_name);
    let request = PermissionRequest::new(
        ActorRef::system(),
        action,
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
            action: Some(capability_action_name(request.action).to_string()),
            resource_kind: Some(resource_kind_name(request.scope.kind).to_string()),
            resource: Some(request.scope.pattern.clone()),
        }),
    ));

    match decision {
        PolicyDecision::Allow { .. } => Ok(PreparedToolCall::Execute {
            events,
            call,
            tool,
            decision,
            request,
        }),
        PolicyDecision::Deny { .. } => Ok(PreparedToolCall::Terminal(ToolLoopOutcome {
            status: ToolLoopStatus::Denied,
            turn_state: TurnState::Failed,
            observation: None,
            pending_approval: None,
            events,
        })),
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
                RuntimeEventPayload::ApprovalRequested(approval_requested_event(&pending_approval)),
            ));

            Ok(PreparedToolCall::Terminal(ToolLoopOutcome {
                status: ToolLoopStatus::WaitingApproval,
                turn_state: TurnState::WaitingApproval,
                observation: None,
                pending_approval: Some(pending_approval),
                events,
            }))
        }
    }
}

fn execution_started_event(
    ids: RuntimeIds,
    call: &ToolCallRequest,
    decision: &PolicyDecision,
    request: &PermissionRequest,
) -> RuntimeEvent {
    event(
        ids,
        RuntimeEventPayload::ToolExecutionStarted(ToolExecutionStarted {
            call_id: call.call_id.clone(),
            tool_name: call.tool_name.clone(),
            policy_decision: Some(decision_name(decision).to_string()),
            policy_reason: Some(decision_reason(decision).to_string()),
            action: Some(capability_action_name(request.action).to_string()),
            resource_kind: Some(resource_kind_name(request.scope.kind).to_string()),
            resource: Some(request.scope.pattern.clone()),
            sandbox_profile: Some(sandbox_profile_for_tool(&call.tool_name).to_string()),
        }),
    )
}

fn finish_tool_execution(
    ids: RuntimeIds,
    call: ToolCallRequest,
    mut events: Vec<RuntimeEvent>,
    started_at: Instant,
    execution: Result<ToolOutput, agent_tool::ToolError>,
) -> ToolLoopOutcome {
    match execution {
        Ok(output) => {
            let observation = observation_from_output(&output.value);
            events.push(event(
                ids,
                RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
                    call_id: call.call_id.clone(),
                    output_json: output.value,
                    elapsed_ms: Some(elapsed_ms(started_at.elapsed())),
                }),
            ));
            events.push(event(
                ids,
                RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
                    call_id: call.call_id,
                    observation: observation.clone(),
                }),
            ));
            ToolLoopOutcome {
                status: ToolLoopStatus::Completed,
                turn_state: TurnState::Observed,
                observation: Some(observation),
                pending_approval: None,
                events,
            }
        }
        Err(error) => {
            events.push(event(
                ids,
                RuntimeEventPayload::ToolExecutionFailed(ToolExecutionFailed {
                    call_id: call.call_id,
                    error_code: "tool_execution_failed".to_string(),
                    message: error.to_string(),
                    recoverable: true,
                    elapsed_ms: Some(elapsed_ms(started_at.elapsed())),
                }),
            ));
            ToolLoopOutcome {
                status: ToolLoopStatus::Failed,
                turn_state: TurnState::Failed,
                observation: None,
                pending_approval: None,
                events,
            }
        }
    }
}

fn elapsed_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Stable sandbox profile labels attached to tool execution events.
/// Built-in workspace tools always run under path confinement; shell adds
/// env allowlisting + timeout. Non-workspace tools are labeled `unrestricted`
/// so UI/audit never pretends a sandbox applied.
#[must_use]
pub fn sandbox_profile_for_tool(tool_name: &str) -> &'static str {
    match tool_name {
        // Shell prefers macOS seatbelt process confinement when available.
        "workspace.shell_command" => "workspace_seatbelt",
        "workspace.read_file"
        | "workspace.write_file"
        | "workspace.edit_file"
        | "workspace.search_text"
        | "workspace.git_status"
        | "workspace.git_diff" => "workspace_fs",
        name if name.starts_with("workspace.") => "workspace",
        name if name.starts_with("mcp.") => "mcp",
        _ => "unrestricted",
    }
}

/// Map tool names to policy capability actions so write/shell are not
/// silently classified as Read.
#[must_use]
pub fn capability_action_for_tool(tool_name: &str) -> agent_policy::CapabilityAction {
    match tool_name {
        "workspace.read_file" | "workspace.git_status" | "workspace.git_diff" => {
            agent_policy::CapabilityAction::Read
        }
        "workspace.search_text" => agent_policy::CapabilityAction::Search,
        // File mutations and shell share Execute; policy rules decide Allow vs Ask.
        "workspace.write_file" | "workspace.edit_file" | "workspace.shell_command" => {
            agent_policy::CapabilityAction::Execute
        }
        name if name.starts_with("mcp.") => agent_policy::CapabilityAction::Network,
        _ => agent_policy::CapabilityAction::Read,
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

#[cfg(test)]
mod tests {
    use super::{capability_action_for_tool, sandbox_profile_for_tool};
    use agent_policy::CapabilityAction;

    #[test]
    fn capability_action_matches_tool_semantics() {
        assert_eq!(
            capability_action_for_tool("workspace.read_file"),
            CapabilityAction::Read
        );
        assert_eq!(
            capability_action_for_tool("workspace.search_text"),
            CapabilityAction::Search
        );
        assert_eq!(
            capability_action_for_tool("workspace.shell_command"),
            CapabilityAction::Execute
        );
        assert_eq!(
            capability_action_for_tool("workspace.edit_file"),
            CapabilityAction::Execute
        );
        assert_eq!(
            capability_action_for_tool("mcp.filesystem.read_file"),
            CapabilityAction::Network
        );
    }

    #[test]
    fn sandbox_profile_labels_are_specific() {
        assert_eq!(
            sandbox_profile_for_tool("workspace.read_file"),
            "workspace_fs"
        );
        assert_eq!(
            sandbox_profile_for_tool("workspace.shell_command"),
            "workspace_seatbelt"
        );
        assert_eq!(sandbox_profile_for_tool("mcp.x"), "mcp");
        assert_eq!(sandbox_profile_for_tool("custom.tool"), "unrestricted");
    }
}
