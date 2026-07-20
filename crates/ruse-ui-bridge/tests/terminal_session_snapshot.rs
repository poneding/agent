use ruse_core::{
    RepoGitState, TerminalSessionDiagnostic, TerminalSessionDiagnosticSeverity,
    TerminalSessionDirectoryReport, TerminalSessionRecord, build_terminal_session_directory,
};
use ruse_types::{
    ClientKind, ConversationId, CorrelationId, SchemaVersion, Timestamp, WorkspaceId,
};
use ruse_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame, UiTerminalSessionSnapshot,
    encode_json_frame, validate_ui_command,
};
use time::macros::datetime;

fn id<T>(value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    value.parse().expect("valid deterministic id")
}

fn ts(day: u8) -> Timestamp {
    Timestamp::from_utc(
        datetime!(2026-07-01 00:00:00 UTC) + time::Duration::days(i64::from(day - 1)),
    )
}

fn record(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    title: &str,
    last_event_at: Timestamp,
) -> TerminalSessionRecord {
    TerminalSessionRecord {
        workspace_id,
        conversation_id,
        workspace_root_display: format!("workspace-{conversation_id}"),
        title: title.to_string(),
        client_kind: ClientKind::Cli,
        client_version: "0.1.0".to_string(),
        provider_id: "fake".to_string(),
        model_id: "fake-scripted".to_string(),
        git_state: RepoGitState::Clean,
        last_event_at,
        pending_approval_count: 0,
        running_command_count: 0,
        task_count: 0,
        failed: false,
    }
}

fn command(workspace_id: WorkspaceId, conversation_id: ConversationId) -> UiCommandEnvelope {
    UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Tui,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::SwitchSession {
            workspace_id,
            conversation_id,
        },
    }
}

#[test]
fn terminal_session_snapshot_serializes_stable_shape() {
    let first_workspace = id::<WorkspaceId>("00000000-0000-0000-0000-000000000001");
    let first_conversation = id::<ConversationId>("00000000-0000-0000-0000-000000000011");
    let second_workspace = id::<WorkspaceId>("00000000-0000-0000-0000-000000000002");
    let second_conversation = id::<ConversationId>("00000000-0000-0000-0000-000000000022");
    let mut first = record(first_workspace, first_conversation, "waiting", ts(2));
    first.pending_approval_count = 1;
    first.git_state = RepoGitState::Dirty;
    let second = record(second_workspace, second_conversation, "idle", ts(1));
    let report = build_terminal_session_directory(vec![second, first]).expect("directory");

    let snapshot = UiTerminalSessionSnapshot::from_report(&report).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(value["sessions"].as_array().expect("sessions").len(), 2);
    assert_eq!(
        value["sessions"][0]["conversation_id"],
        first_conversation.to_string()
    );
    assert_eq!(value["sessions"][0]["title"], "waiting");
    assert_eq!(value["sessions"][0]["status"], "waiting_approval");
    assert_eq!(value["sessions"][0]["client_kind"], "cli");
    assert_eq!(value["sessions"][0]["git_state"], "dirty");
    assert_eq!(value["sessions"][0]["pending_approval_count"], 1);
    assert_eq!(
        value["diagnostics"].as_array().expect("diagnostics").len(),
        0
    );
}

#[test]
fn terminal_session_snapshot_bounds_and_redacts_diagnostics() {
    let report = TerminalSessionDirectoryReport {
        sessions: Vec::new(),
        diagnostics: vec![TerminalSessionDiagnostic {
            severity: TerminalSessionDiagnosticSeverity::Warning,
            code: "diagnostic-secret-code".to_string(),
            message: format!("sk-test-secret {}", "x".repeat(200)),
            recoverable: true,
        }],
    };

    let snapshot = UiTerminalSessionSnapshot::from_report(&report).expect("snapshot");
    let rendered = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("diagnostic-secret-code"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(snapshot.diagnostics[0].code.chars().count() <= 120);
    assert!(snapshot.diagnostics[0].message.chars().count() <= 120);
}

#[test]
fn terminal_session_snapshot_json_frame_is_single_line() {
    let report = build_terminal_session_directory(Vec::new()).expect("directory");
    let snapshot = UiTerminalSessionSnapshot::from_report(&report).expect("snapshot");

    let line = encode_json_frame(&UiJsonFrame::TerminalSessionSnapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::TerminalSessionSnapshot(_)));
}

#[test]
fn switch_session_command_validates_target_ids() {
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    let envelope = command(workspace_id, conversation_id);

    validate_ui_command(&envelope).expect("valid switch command");

    let nil_workspace = id::<WorkspaceId>("00000000-0000-0000-0000-000000000000");
    let error =
        validate_ui_command(&command(nil_workspace, conversation_id)).expect_err("nil rejected");
    assert!(error.to_string().contains("workspace_id"));
}
