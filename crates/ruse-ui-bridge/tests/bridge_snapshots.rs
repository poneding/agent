use ruse_types::{
    ActorRef, ClientKind, ConversationId, CorrelationId, EventId, EventSeq, ModelOutputDelta,
    PromptLayerHashRecord, PromptSnapshotRecorded, RuntimeEvent, RuntimeEventPayload,
    SchemaVersion, Timestamp, ToolCallProposed, TurnId, WorkspaceId,
};
use ruse_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiEventPayload, UiJsonFrame,
    normalize_runtime_event,
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

#[test]
fn prompt_snapshot_event_normalizes_to_ui_prompt_summary() {
    let event = runtime_event(RuntimeEventPayload::PromptSnapshotRecorded(
        PromptSnapshotRecorded {
            profile_id: "default".to_string(),
            profile_version: 1,
            bundle_hash: "fnv1a64:bundle".to_string(),
            layer_hashes: vec![PromptLayerHashRecord {
                layer_id: "system".to_string(),
                source_hash: "fnv1a64:source".to_string(),
                content_hash: "fnv1a64:content".to_string(),
            }],
            diagnostic_count: 2,
            message_count: 4,
            estimated_tokens: 140,
        },
    ));

    let ui_event = normalize_runtime_event(&event);
    let UiEventPayload::PromptSummary(summary) = ui_event.payload else {
        panic!("expected prompt summary payload");
    };

    assert_eq!(summary.profile_id, "default");
    assert_eq!(summary.profile_version, 1);
    assert_eq!(summary.display_name, "default");
    assert_eq!(summary.bundle_hash, "fnv1a64:bundle");
    assert_eq!(summary.estimated_tokens, 140);
    assert_eq!(summary.message_count, 4);
    assert_eq!(summary.layers.len(), 1);
    assert_eq!(summary.layers[0].layer_id, "system");
    assert_eq!(
        summary.layers[0].source_hash.as_deref(),
        Some("fnv1a64:source")
    );
    assert_eq!(
        summary.layers[0].content_hash.as_deref(),
        Some("fnv1a64:content")
    );
    assert_eq!(summary.layers[0].content_preview, None);
    assert!(
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "prompt.snapshot.partial_metadata")
    );
    assert!(
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "prompt.snapshot.diagnostics_recorded")
    );
}
