use agent_types::{
    ActorRef, ClientKind, ConversationId, CorrelationId, EventId, EventSeq, ModelOutputDelta,
    RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed, TurnId,
    WorkspaceId,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame, normalize_runtime_event,
};
use serde_json::{Value, json};
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

fn fixture(contents: &str) -> Value {
    serde_json::from_str(contents).expect("fixture is valid json")
}

#[test]
fn bridge_snapshots_match_model_delta_event_fixture() {
    let event = runtime_event(RuntimeEventPayload::ModelOutputDelta(ModelOutputDelta {
        delta: "hello".to_string(),
    }));

    let value = serde_json::to_value(normalize_runtime_event(&event)).expect("serialize ui event");

    assert_eq!(
        value,
        fixture(include_str!("fixtures/m06_model_delta_event.json"))
    );
}

#[test]
fn bridge_snapshots_match_start_turn_command_fixture() {
    let command = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000100"),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::StartTurn {
            workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
            conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
            prompt: "Summarize the project".to_string(),
        },
    };

    let value = serde_json::to_value(command).expect("serialize command");

    assert_eq!(
        value,
        fixture(include_str!("fixtures/m06_start_turn_command.json"))
    );
}

#[test]
fn bridge_snapshots_match_done_frame_fixture() {
    let value =
        serde_json::to_value(UiJsonFrame::Done { event_count: 3 }).expect("serialize done frame");

    assert_eq!(value, fixture(include_str!("fixtures/m06_done_frame.json")));
}

#[test]
fn bridge_snapshots_redact_tool_call_input_from_ui_event() {
    let event = runtime_event(RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
        call_id: "call-1".to_string(),
        tool_name: "shell_command".to_string(),
        input_json: json!({
            "command": "echo ok",
            "api_token": "secret-token"
        }),
    }));

    let value = serde_json::to_value(normalize_runtime_event(&event)).expect("serialize ui event");
    let serialized = value.to_string();

    assert!(!serialized.contains("secret-token"));
    assert!(!serialized.contains("api_token"));
    assert_eq!(
        value["payload"],
        json!({
            "type": "tool_call",
            "data": {
                "call_id": "call-1",
                "tool_name": "shell_command"
            }
        })
    );
}
