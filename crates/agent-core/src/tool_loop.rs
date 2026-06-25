use agent_policy::{PermissionRequest, PolicyDecision, PolicyEngine, ResourceKind, ResourceScope};
use agent_tool::{ToolInput, ToolName, ToolRegistry};
use agent_types::{
    ActorRef, PermissionDecisionRecorded, RuntimeEvent, RuntimeEventPayload, SchemaVersion,
    Timestamp, ToolCallProposed, ToolExecutionCompleted, ToolExecutionFailed, ToolExecutionStarted,
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
                            events,
                        })
                    }
                }
            }
            PolicyDecision::Deny { .. } => Ok(ToolLoopOutcome {
                status: ToolLoopStatus::Denied,
                turn_state: TurnState::Failed,
                observation: None,
                events,
            }),
            PolicyDecision::Ask { .. } => Ok(ToolLoopOutcome {
                status: ToolLoopStatus::WaitingApproval,
                turn_state: TurnState::WaitingApproval,
                observation: None,
                events,
            }),
        }
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
