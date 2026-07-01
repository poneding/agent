use agent_types::{
    ClientKind, ConversationId, CorrelationId, EventId, EventSeq, SchemaVersion, Timestamp,
    WorkspaceId,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiEventEnvelope, UiEventPayload, UiJsonFrame,
    encode_json_frame, validate_ui_command,
};
use serde_json::json;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn workspace_id() -> WorkspaceId {
    id("00000000-0000-0000-0000-000000000001")
}

fn conversation_id() -> ConversationId {
    id("00000000-0000-0000-0000-000000000002")
}

fn command_id() -> CorrelationId {
    id("00000000-0000-0000-0000-000000000003")
}

fn start_turn_command(prompt: &str) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: command_id(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::StartTurn {
            workspace_id: workspace_id(),
            conversation_id: conversation_id(),
            prompt: prompt.to_string(),
        },
    }
}

fn event_frame(payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: SchemaVersion::current(),
        event_id: id::<EventId>("00000000-0000-0000-0000-000000000004"),
        seq: Some(EventSeq::new(1).expect("valid sequence")),
        occurred_at: Timestamp::now_utc(),
        conversation_id: conversation_id(),
        turn_id: None,
        task_id: None,
        correlation_id: command_id(),
        payload,
    })
}

#[test]
fn desktop_start_turn_command_serializes_and_validates() {
    let envelope = start_turn_command("Summarize this workspace");

    validate_ui_command(&envelope).expect("valid start turn command");

    let value = serde_json::to_value(envelope).expect("serialize command");
    assert_eq!(
        value,
        json!({
            "schema_version": 1,
            "command_id": "00000000-0000-0000-0000-000000000003",
            "client": {
                "kind": "desktop",
                "version": "0.1.0"
            },
            "command": {
                "type": "start_turn",
                "data": {
                    "workspace_id": "00000000-0000-0000-0000-000000000001",
                    "conversation_id": "00000000-0000-0000-0000-000000000002",
                    "prompt": "Summarize this workspace"
                }
            }
        })
    );
}

#[test]
fn desktop_start_turn_rejects_empty_prompt() {
    let error = validate_ui_command(&start_turn_command("   ")).expect_err("empty prompt rejected");

    assert!(
        error
            .to_string()
            .contains("start_turn prompt cannot be empty")
    );
}

#[test]
fn transcript_relevant_frames_encode_as_single_line_json() {
    let frames = [
        event_frame(UiEventPayload::UserMessage {
            message: "hello".to_string(),
        }),
        event_frame(UiEventPayload::ModelDelta {
            delta: "hi".to_string(),
        }),
        UiJsonFrame::Done { event_count: 2 },
        UiJsonFrame::Error {
            code: "desktop.bridge.disconnected".to_string(),
            message: "desktop bridge session is not connected".to_string(),
            recoverable: true,
        },
    ];

    for frame in frames {
        let line = encode_json_frame(&frame).expect("encode frame");
        assert!(line.ends_with('\n'));
        assert_eq!(line.matches('\n').count(), 1);
        serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("frame parses");
    }
}
