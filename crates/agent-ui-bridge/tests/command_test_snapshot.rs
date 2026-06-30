use agent_core::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandTestLoopReport,
};
use agent_types::{ConversationId, WorkspaceId};
use agent_ui_bridge::{UiCommandTestSnapshot, UiJsonFrame, encode_json_frame};

fn output(text: &str) -> BoundedCommandOutput {
    BoundedCommandOutput {
        text: text.to_string(),
        byte_count: text.len().try_into().expect("byte count"),
        truncated: false,
        redacted: false,
    }
}

fn report() -> CommandTestLoopReport {
    CommandTestLoopReport {
        workspace_id: WorkspaceId::new(),
        conversation_id: ConversationId::new(),
        command: CommandIntentSummary {
            display: "cargo test -p agent-core command_test_loop".to_string(),
            cwd: ".".to_string(),
            classify_as_test: true,
        },
        status: CommandExecutionStatus::Failed,
        classification: CommandFailureClassification::TestFailure,
        exit_code: Some(101),
        stdout: output("running tests\n"),
        stderr: output("test failed\n"),
        timed_out: false,
        cancellation: CommandCancellationState {
            cancellable: true,
            reason: "command can be cancelled before completion".to_string(),
        },
        observation: CommandObservationSummary {
            message: "command observation: status=101".to_string(),
            truncated: false,
            redacted: false,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn command_test_snapshot_serializes_safe_shape() {
    let snapshot = UiCommandTestSnapshot::from_report(&report()).expect("snapshot");
    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value["command"],
        "cargo test -p agent-core command_test_loop"
    );
    assert_eq!(value["cwd"], ".");
    assert_eq!(value["classify_as_test"], true);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["classification"], "test_failure");
    assert_eq!(value["exit_code"], 101);
    assert_eq!(value["timed_out"], false);
    assert_eq!(value["cancellable"], true);
    assert_eq!(value["stdout"], "running tests");
    assert_eq!(value["stderr"], "test failed");
}

#[test]
fn command_test_snapshot_redacts_secret_like_output() {
    let mut report = report();
    report.stdout.text = "sk-test-secret api_key password".to_string();
    report.stdout.redacted = true;
    report.observation.message = "stdout sk-test-secret".to_string();
    report.observation.redacted = true;

    let snapshot = UiCommandTestSnapshot::from_report(&report).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize");

    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("password"));
    assert!(serialized.contains("[redacted]"));
    assert!(snapshot.redacted);
}

#[test]
fn command_test_snapshot_marks_truncation() {
    let mut report = report();
    report.stdout.text = "x".repeat(5_000);
    report.stdout.byte_count = 5_000;
    report.stdout.truncated = true;
    report.observation.truncated = true;

    let snapshot = UiCommandTestSnapshot::from_report(&report).expect("snapshot");

    assert!(snapshot.truncated);
    assert_eq!(snapshot.stdout.len(), 4_000);
    assert_eq!(snapshot.stdout_byte_count, 5_000);
}

#[test]
fn command_test_snapshot_json_frame_is_single_line() {
    let snapshot = UiCommandTestSnapshot::from_report(&report()).expect("snapshot");

    let line = encode_json_frame(&UiJsonFrame::CommandTestSnapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::CommandTestSnapshot(_)));
}
