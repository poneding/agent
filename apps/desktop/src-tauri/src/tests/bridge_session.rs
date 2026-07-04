use crate::{
    DesktopCommandError, DesktopRuntimeSession, DesktopRuntimeState, DesktopStartSessionRequest,
};
use agent_types::{ClientKind, ConversationId, CorrelationId, SchemaVersion, WorkspaceId};
use agent_ui_bridge::{UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame};

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn start_request() -> DesktopStartSessionRequest {
    DesktopStartSessionRequest {
        workspace_path: Some("/tmp/agent".to_string()),
        client_version: "0.1.0".to_string(),
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
