use agent_types::{
    CheckpointId, ClientKind, ConversationId, CorrelationId, EventId, EventSeq, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, TaskId, Timestamp, TurnId, TurnState, WorkspaceTrustLevel,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UiEventEnvelope {
    pub schema_version: SchemaVersion,
    pub event_id: EventId,
    pub seq: Option<EventSeq>,
    pub occurred_at: Timestamp,
    pub conversation_id: ConversationId,
    pub turn_id: Option<TurnId>,
    pub task_id: Option<TaskId>,
    pub correlation_id: CorrelationId,
    pub payload: UiEventPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiEventPayload {
    Workspace {
        root_path: String,
        trust_level: String,
    },
    Session {
        client_kind: String,
        client_version: String,
    },
    UserMessage {
        message: String,
    },
    TurnState {
        state: TurnState,
        reason: String,
    },
    ModelDelta {
        delta: String,
    },
    FinalResponse {
        message: String,
    },
    ToolCall {
        call_id: String,
        tool_name: String,
    },
    ToolResult {
        call_id: String,
        ok: bool,
        summary: String,
    },
    Permission {
        request_id: String,
        decision: String,
        reason: String,
    },
    Checkpoint {
        checkpoint_id: CheckpointId,
        label: String,
    },
    Diagnostic {
        level: UiDiagnosticLevel,
        code: String,
        message: String,
        recoverable: bool,
    },
    Progress {
        message: String,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiDiagnosticLevel {
    Warning,
    Error,
}

#[must_use]
pub fn normalize_runtime_event(event: &RuntimeEvent) -> UiEventEnvelope {
    UiEventEnvelope {
        schema_version: event.schema_version,
        event_id: event.id,
        seq: event.seq,
        occurred_at: event.occurred_at,
        conversation_id: event.conversation_id,
        turn_id: event.turn_id,
        task_id: event.task_id,
        correlation_id: event.correlation_id,
        payload: normalize_payload(&event.payload),
    }
}

fn normalize_payload(payload: &RuntimeEventPayload) -> UiEventPayload {
    match payload {
        RuntimeEventPayload::WorkspaceOpened(opened) => UiEventPayload::Workspace {
            root_path: opened.root_path.clone(),
            trust_level: trust_level_name(&opened.trust_level).to_string(),
        },
        RuntimeEventPayload::SessionStarted(started) => UiEventPayload::Session {
            client_kind: client_kind_name(&started.client_kind).to_string(),
            client_version: started.client_version.clone(),
        },
        RuntimeEventPayload::UserInputReceived(input) => UiEventPayload::UserMessage {
            message: input.message.clone(),
        },
        RuntimeEventPayload::TurnStateChanged(changed) => UiEventPayload::TurnState {
            state: changed.to,
            reason: changed.reason.clone(),
        },
        RuntimeEventPayload::ModelOutputDelta(delta) => UiEventPayload::ModelDelta {
            delta: delta.delta.clone(),
        },
        RuntimeEventPayload::FinalResponseProduced(response) => UiEventPayload::FinalResponse {
            message: response.message.clone(),
        },
        RuntimeEventPayload::ToolCallProposed(call) => UiEventPayload::ToolCall {
            call_id: call.call_id.clone(),
            tool_name: call.tool_name.clone(),
        },
        RuntimeEventPayload::ToolExecutionCompleted(completed) => UiEventPayload::ToolResult {
            call_id: completed.call_id.clone(),
            ok: true,
            summary: "completed".to_string(),
        },
        RuntimeEventPayload::ToolExecutionFailed(failed) => UiEventPayload::ToolResult {
            call_id: failed.call_id.clone(),
            ok: false,
            summary: failed.message.clone(),
        },
        RuntimeEventPayload::PermissionDecisionRecorded(decision) => UiEventPayload::Permission {
            request_id: decision.request_id.clone(),
            decision: decision.decision.clone(),
            reason: decision.reason.clone(),
        },
        RuntimeEventPayload::CheckpointRecorded(checkpoint) => UiEventPayload::Checkpoint {
            checkpoint_id: checkpoint.checkpoint_id,
            label: checkpoint.label.clone(),
        },
        RuntimeEventPayload::RuntimeWarning(warning) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Warning,
            code: warning.code.clone(),
            message: warning.message.clone(),
            recoverable: true,
        },
        RuntimeEventPayload::RuntimeError(error) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: error.code.clone(),
            message: error.message.clone(),
            recoverable: error.recoverable,
        },
        RuntimeEventPayload::TurnStarted(started) => UiEventPayload::Progress {
            message: started.input_summary.clone(),
        },
        RuntimeEventPayload::ContextAssembled(context) => UiEventPayload::Progress {
            message: format!(
                "context assembled: {} messages, {} estimated tokens",
                context.message_count, context.estimated_tokens
            ),
        },
        RuntimeEventPayload::ModelStreamStarted(started) => UiEventPayload::Progress {
            message: format!(
                "model stream started: {}/{}",
                started.provider, started.model
            ),
        },
        RuntimeEventPayload::ModelStreamCompleted(completed) => UiEventPayload::Progress {
            message: format!("model stream completed: {:?}", completed.finish_reason),
        },
        RuntimeEventPayload::TurnCompleted(completed) => UiEventPayload::Progress {
            message: completed.summary.clone(),
        },
        RuntimeEventPayload::TurnFailed(failed) => UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: failed.error.code.clone(),
            message: failed.error.message.clone(),
            recoverable: failed.error.recoverable,
        },
        RuntimeEventPayload::TaskStateChanged(changed) => UiEventPayload::Progress {
            message: format!("task state changed: {:?} -> {:?}", changed.from, changed.to),
        },
        RuntimeEventPayload::ToolExecutionStarted(started) => UiEventPayload::Progress {
            message: format!(
                "tool execution started: {} {}",
                started.tool_name, started.call_id
            ),
        },
        RuntimeEventPayload::ToolObservationRecorded(observed) => UiEventPayload::Progress {
            message: format!("tool observation recorded: {}", observed.call_id),
        },
        RuntimeEventPayload::ReplayStarted(replay) => UiEventPayload::Progress {
            message: format!("replay started: {}", replay.source_turn_id),
        },
        RuntimeEventPayload::ReplayCompleted(replay) => UiEventPayload::Progress {
            message: format!("replay completed: {} events", replay.event_count),
        },
        RuntimeEventPayload::ConversationStarted(started) => UiEventPayload::Progress {
            message: started.title.clone(),
        },
    }
}

fn trust_level_name(level: &WorkspaceTrustLevel) -> &'static str {
    match level {
        WorkspaceTrustLevel::Trusted => "trusted",
        WorkspaceTrustLevel::Untrusted => "untrusted",
    }
}

fn client_kind_name(kind: &ClientKind) -> &'static str {
    match kind {
        ClientKind::Cli => "cli",
        ClientKind::Desktop => "desktop",
        ClientKind::Api => "api",
        ClientKind::Test => "test",
    }
}
