use agent_types::{
    ActorRef, ConversationId, CorrelationId, EventId, EventSeq, FinalResponseProduced,
    ModelOutputDelta, ModelReasoningSummaryDelta, RuntimeError, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, Timestamp, ToolCallProposed, ToolExecutionCompleted, ToolExecutionStarted,
    ToolObservationRecorded, TurnId, WorkspaceId,
};
use agent_ui_bridge::{
    UiDiagnosticLevel, UiEventPayload, UiToolPolicySummary, UiToolSandboxSummary,
    normalize_runtime_event,
};
use serde_json::json;
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn runtime_event(payload: RuntimeEventPayload) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: id::<EventId>("00000000-0000-0000-0000-000000000005"),
        seq: Some(EventSeq::new(7).expect("valid sequence")),
        occurred_at: Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC)),
        workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
        conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
        turn_id: Some(id::<TurnId>("00000000-0000-0000-0000-000000000003")),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000004"),
        causation_id: None,
        payload,
    }
}

#[test]
fn ui_event_schema_serializes_model_delta_shape() {
    let event = runtime_event(RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta {
        delta: "hello".to_string(),
    }));

    let value = serde_json::to_value(normalize_runtime_event(&event)).expect("serialize ui event");

    assert_eq!(
        value,
        json!({
            "schema_version": 1,
            "event_id": "00000000-0000-0000-0000-000000000005",
            "seq": 7,
            "occurred_at": "2026-06-25T00:00:00Z",
            "conversation_id": "00000000-0000-0000-0000-000000000002",
            "turn_id": "00000000-0000-0000-0000-000000000003",
            "task_id": null,
            "correlation_id": "00000000-0000-0000-0000-000000000004",
            "payload": {
                "type": "model_delta",
                "data": {
                    "delta": "hello"
                }
            }
        })
    );
}

#[test]
fn ui_event_schema_serializes_reasoning_summary_delta_shape() {
    let event = runtime_event(RuntimeEventPayload::ModelReasoningSummaryDelta(
        ModelReasoningSummaryDelta {
            delta: "checking context".to_string(),
        },
    ));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ReasoningSummaryDelta {
            delta: "checking context".to_string(),
        }
    );
}

#[test]
fn ui_event_schema_normalizes_final_response() {
    let event = runtime_event(RuntimeEventPayload::FinalResponseProduced(
        FinalResponseProduced {
            message: "ready".to_string(),
        },
    ));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::FinalResponse {
            message: "ready".to_string()
        }
    );
    assert_eq!(ui_event.event_id, event.id);
    assert_eq!(ui_event.conversation_id, event.conversation_id);
}

#[test]
fn ui_event_schema_excludes_raw_tool_input() {
    let event = runtime_event(RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
        call_id: "call-1".to_string(),
        tool_name: "shell_command".to_string(),
        input_json: json!({
            "command": "echo ok",
            "api_token": "secret-token"
        }),
    }));

    let ui_event = normalize_runtime_event(&event);
    let serialized = serde_json::to_string(&ui_event).expect("serialize ui event");

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ToolCall {
            call_id: "call-1".to_string(),
            tool_name: "shell_command".to_string(),
            preview: None,
        }
    );
    assert!(!serialized.contains("secret-token"));
    assert!(!serialized.contains("api_token"));
}

#[test]
fn ui_event_schema_includes_redacted_builtin_tool_preview() {
    let event = runtime_event(RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
        call_id: "call-1".to_string(),
        tool_name: "workspace.shell_command".to_string(),
        input_json: json!({
            "program": "echo",
            "args": ["ok", "api_token=secret-token"]
        }),
    }));

    let ui_event = normalize_runtime_event(&event);
    let serialized = serde_json::to_string(&ui_event).expect("serialize ui event");

    assert_eq!(
        ui_event.payload,
        UiEventPayload::ToolCall {
            call_id: "call-1".to_string(),
            tool_name: "workspace.shell_command".to_string(),
            preview: Some("run echo ok [redacted]".to_string()),
        }
    );
    assert!(!serialized.contains("secret-token"));
    assert!(!serialized.contains("api_token"));
}

#[test]
fn ui_event_schema_preserves_tool_lifecycle_events() {
    let started = normalize_runtime_event(&runtime_event(
        RuntimeEventPayload::ToolExecutionStarted(ToolExecutionStarted {
            call_id: "call-1".to_string(),
            tool_name: "shell_command".to_string(),
            policy_decision: Some("allow".to_string()),
            policy_reason: Some("read-only path".to_string()),
            action: Some("read".to_string()),
            resource_kind: Some("tool".to_string()),
            resource: Some("shell_command".to_string()),
            sandbox_profile: Some("workspace".to_string()),
        }),
    ));
    let observed = normalize_runtime_event(&runtime_event(
        RuntimeEventPayload::ToolObservationRecorded(ToolObservationRecorded {
            call_id: "call-1".to_string(),
            observation: "stdout: ok".to_string(),
        }),
    ));

    assert_eq!(
        started.payload,
        UiEventPayload::ToolStarted {
            call_id: "call-1".to_string(),
            tool_name: "shell_command".to_string(),
            policy: Some(UiToolPolicySummary {
                decision: "allow".to_string(),
                reason: "read-only path".to_string(),
                action: "read".to_string(),
                resource_kind: "tool".to_string(),
                resource: "shell_command".to_string(),
            }),
            sandbox: Some(UiToolSandboxSummary {
                profile: "workspace".to_string(),
            })
        }
    );
    assert_eq!(
        observed.payload,
        UiEventPayload::ToolObservation {
            call_id: "call-1".to_string(),
            observation: "stdout: ok".to_string()
        }
    );
}

#[test]
fn ui_event_schema_summarizes_tool_completion_metadata() {
    let completed = normalize_runtime_event(&runtime_event(
        RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
            call_id: "call-1".to_string(),
            output_json: json!({
                "status": 101,
                "stdout": "running tests",
                "stderr": "test failed",
                "timed_out": false
            }),
            elapsed_ms: Some(42),
        }),
    ));

    assert_eq!(
        completed.payload,
        UiEventPayload::ToolResult {
            call_id: "call-1".to_string(),
            ok: true,
            summary: "exit=101 stdout=running tests stderr=test failed".to_string(),
            elapsed_ms: Some(42),
        }
    );
}

#[test]
fn ui_event_schema_normalizes_runtime_error_as_diagnostic() {
    let event = runtime_event(RuntimeEventPayload::RuntimeError(RuntimeError {
        code: "model_error".to_string(),
        message: "provider failed".to_string(),
        recoverable: false,
    }));

    let ui_event = normalize_runtime_event(&event);

    assert_eq!(
        ui_event.payload,
        UiEventPayload::Diagnostic {
            level: UiDiagnosticLevel::Error,
            code: "model_error".to_string(),
            message: "provider failed".to_string(),
            recoverable: false
        }
    );
}
