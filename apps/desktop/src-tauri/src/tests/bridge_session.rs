use std::fs;

use ruse_storage::SqliteEventStore;

use crate::{
    DesktopCommandError, DesktopRuntimeSession, DesktopRuntimeState, DesktopStartSessionRequest,
};
use ruse_types::{
    ActorRef, ClientKind, ConversationId, CorrelationId, EventId, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, UserInputReceived, WorkspaceId,
};
use ruse_ui_bridge::{UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame};

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn user_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    message: &str,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id,
        conversation_id,
        turn_id: None,
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload: RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    }
}

fn start_request() -> DesktopStartSessionRequest {
    DesktopStartSessionRequest {
        workspace_path: Some("/tmp/agent".to_string()),
        client_version: "0.1.0".to_string(),
    }
}

fn switch_session_command(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000101"),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::SwitchSession {
            workspace_id,
            conversation_id,
        },
    }
}

fn start_turn_command(prompt: &str) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: id::<CorrelationId>("00000000-0000-0000-0000-000000000100"),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::StartTurn {
            workspace_id: id::<WorkspaceId>("00000000-0000-0000-0000-000000000001"),
            conversation_id: id::<ConversationId>("00000000-0000-0000-0000-000000000002"),
            prompt: prompt.to_string(),
        },
    }
}

#[test]
fn start_session_creates_connected_snapshot_and_duplicate_start_reuses_it() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);

    let first = session.start(start_request()).expect("session starts");
    let second = session
        .start(DesktopStartSessionRequest {
            workspace_path: Some("/other/path".to_string()),
            client_version: "0.1.0".to_string(),
        })
        .expect("duplicate start is idempotent");

    assert_eq!(first.session_id, second.session_id);
    assert_eq!(first.workspace_id, second.workspace_id);
    assert_eq!(first.conversation_id, second.conversation_id);
    assert_eq!(first.state, DesktopRuntimeState::Connected);
    assert_eq!(second.state, DesktopRuntimeState::Connected);
}

#[test]
fn stop_session_is_idempotent_after_start_or_before_start() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);

    let initial_stop = session.stop().expect("stop before start is allowed");
    assert_eq!(initial_stop.state, DesktopRuntimeState::Disconnected);

    session.start(start_request()).expect("session starts");
    let stopped = session.stop().expect("session stops");
    let stopped_again = session.stop().expect("second stop is allowed");

    assert_eq!(stopped.state, DesktopRuntimeState::Disconnected);
    assert_eq!(stopped_again.state, DesktopRuntimeState::Disconnected);
    assert_eq!(stopped.session_id, stopped_again.session_id);
}

#[test]
fn send_ui_command_validates_before_acknowledging() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session.start(start_request()).expect("session starts");
    session.clear_frames_for_tests();

    let ack = session
        .send_ui_command(start_turn_command("Summarize this workspace"))
        .expect("valid command is accepted");

    assert!(ack.accepted);
    assert_eq!(ack.state, DesktopRuntimeState::Connected);
    assert_eq!(ack.command_id, "00000000-0000-0000-0000-000000000100");
    assert_eq!(ack.pending_frame_count, 2);

    let error = session
        .send_ui_command(start_turn_command("   "))
        .expect_err("empty prompt is rejected");
    assert_eq!(error.code, "desktop.bridge.command_invalid");
    assert!(error.message.contains("start_turn prompt cannot be empty"));
}

#[test]
fn switch_session_command_updates_desktop_runtime_identity() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session.start(start_request()).expect("session starts");
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();

    let ack = session
        .send_ui_command(switch_session_command(workspace_id, conversation_id))
        .expect("switch session accepted");
    let identity = session.workspace_snapshot_identity();

    assert!(ack.accepted);
    assert_eq!(identity, (workspace_id, Some(conversation_id)));
}

#[test]
fn switch_session_replays_persisted_history_and_lists_generated_title() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session.start(start_request()).expect("session starts");
    session.clear_frames_for_tests();
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    session
        .append_event_for_tests(user_event(
            workspace_id,
            conversation_id,
            "Investigate Desktop session history. Later details",
        ))
        .expect("append history");

    let ack = session
        .send_ui_command(switch_session_command(workspace_id, conversation_id))
        .expect("switch session accepted");
    let frames = session.drain_ui_frames(8).expect("drain replay");
    let conversations = session
        .persisted_conversations()
        .expect("persisted conversations");

    assert_eq!(ack.pending_frame_count, 2);
    assert!(frames.iter().any(|frame| matches!(
        frame,
        UiJsonFrame::Event(event)
            if event.conversation_id == conversation_id
                && matches!(
                    &event.payload,
                    ruse_ui_bridge::UiEventPayload::UserMessage { message }
                        if message == "Investigate Desktop session history. Later details"
                )
    )));
    assert!(
        frames
            .iter()
            .any(|frame| matches!(frame, UiJsonFrame::Done { event_count: 1 }))
    );
    assert_eq!(conversations.len(), 1);
    assert_eq!(
        conversations[0].title,
        "Investigate Desktop session history"
    );
}

#[test]
fn desktop_start_turn_persists_user_history_across_store_reopen() {
    let path =
        std::env::temp_dir().join(format!("ruse-desktop-bridge-{}.sqlite3", EventId::new()));
    let workspace_id = id::<WorkspaceId>("00000000-0000-0000-0000-000000000001");
    let conversation_id = id::<ConversationId>("00000000-0000-0000-0000-000000000002");
    let prompt = "Persist this Desktop conversation across restart";

    {
        let store = SqliteEventStore::open(&path).expect("open event store");
        let mut session = DesktopRuntimeSession::new_with_event_store_for_tests(8, store);
        session.start(start_request()).expect("session starts");
        session.clear_frames_for_tests();

        let ack = session
            .send_ui_command(start_turn_command(prompt))
            .expect("turn accepted");
        let frames = session.drain_ui_frames(8).expect("drain start turn");

        assert_eq!(ack.pending_frame_count, 2);
        assert!(frames.iter().any(|frame| matches!(
            frame,
            UiJsonFrame::Event(event)
                if matches!(
                    &event.payload,
                    ruse_ui_bridge::UiEventPayload::UserMessage { message }
                        if message == prompt
                )
        )));
        assert!(
            frames
                .iter()
                .any(|frame| matches!(frame, UiJsonFrame::Done { event_count: 1 }))
        );
    }

    {
        let store = SqliteEventStore::open(&path).expect("reopen event store");
        let mut session = DesktopRuntimeSession::new_with_event_store_for_tests(8, store);
        session.start(start_request()).expect("session restarts");
        session.clear_frames_for_tests();

        let conversations = session.persisted_conversations().expect("list history");
        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].conversation_id, conversation_id);
        assert_eq!(
            conversations[0].title,
            "Persist this Desktop conversation across restart"
        );

        let ack = session
            .send_ui_command(switch_session_command(workspace_id, conversation_id))
            .expect("switch persisted session");
        let frames = session.drain_ui_frames(8).expect("drain replay");
        assert_eq!(ack.pending_frame_count, 2);
        assert!(frames.iter().any(|frame| matches!(
            frame,
            UiJsonFrame::Event(event)
                if matches!(
                    &event.payload,
                    ruse_ui_bridge::UiEventPayload::UserMessage { message }
                        if message == prompt
                )
        )));
        assert!(
            frames
                .iter()
                .any(|frame| matches!(frame, UiJsonFrame::Done { event_count: 1 }))
        );
    }

    let _ = fs::remove_file(path);
}

#[test]
fn send_ui_command_rejects_disconnected_session_without_acknowledgement() {
    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session.start(start_request()).expect("session starts");
    session.stop().expect("session stops");

    let error = session
        .send_ui_command(start_turn_command("Summarize this workspace"))
        .expect_err("disconnected command is rejected");

    assert_eq!(error.code, "desktop.bridge.disconnected");
    assert!(error.message.contains("not connected"));
}

#[test]
fn frame_draining_is_bounded_and_reports_dropped_frames() {
    let mut session = DesktopRuntimeSession::new_for_tests(2);
    session.start(start_request()).expect("session starts");
    session.clear_frames_for_tests();

    session.enqueue_diagnostic_for_tests("one");
    session.enqueue_diagnostic_for_tests("two");
    session.enqueue_diagnostic_for_tests("three");

    let frames = session.drain_ui_frames(10).expect("frames drain");

    assert_eq!(frames.len(), 2);
    assert!(
        frames.iter().any(|frame| matches!(
            frame,
            UiJsonFrame::Error {
                code,
                message,
                recoverable: true,
            } if code == "desktop.bridge.frames_dropped" && message.contains("dropped")
        )),
        "bounded queue should surface a dropped-frame diagnostic"
    );
}

#[test]
fn diagnostics_and_errors_are_redacted_before_serialization() {
    let error = DesktopCommandError::new(
        "desktop.bridge.test",
        "token sk-test api_key password should not leak",
    );
    assert_eq!(
        error.message,
        "[redacted] [redacted] [redacted] [redacted] should not leak"
    );

    let mut session = DesktopRuntimeSession::new_for_tests(8);
    session.start(start_request()).expect("session starts");
    session.clear_frames_for_tests();
    session.enqueue_diagnostic_for_tests("secret token sk-test remains hidden");

    let frames = session.drain_ui_frames(1).expect("frames drain");
    let serialized = serde_json::to_string(&frames).expect("serialize frames");

    assert!(!serialized.contains("sk-test"));
    assert!(!serialized.contains("token"));
    assert!(serialized.contains("[redacted]"));
}
