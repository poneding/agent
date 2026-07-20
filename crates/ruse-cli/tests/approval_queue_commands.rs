use ruse_cli::{
    ApprovalCliCommand, CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation,
};
use ruse_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn approval_queue_command_parses_text_and_json_forms() {
    let text = parse_cli_invocation(["approval", "queue", "--fixture", "empty"])
        .expect("parse text approval queue");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(
        text.command,
        CliCommand::Approval(ApprovalCliCommand::Queue { .. })
    ));

    let json = parse_cli_invocation(["--json", "approval", "queue", "--fixture", "sample"])
        .expect("parse json approval queue");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(
        json.command,
        CliCommand::Approval(ApprovalCliCommand::Queue { .. })
    ));
}

#[test]
fn approval_queue_text_empty_fixture_is_successful() {
    let output = render_cli_invocation(
        parse_cli_invocation(["approval", "queue", "--fixture", "empty"]).expect("parse"),
    );

    assert_eq!(output.status_code, 0);
    assert_eq!(output.stdout, "Approval queue: empty\n");
}

#[test]
fn approval_queue_text_sample_renders_safe_summary() {
    let output = render_cli_invocation(
        parse_cli_invocation(["approval", "queue", "--fixture", "sample"]).expect("parse"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Approval queue: 2 pending"));
    assert!(
        output
            .stdout
            .contains("group command/execute/pending count=1")
    );
    assert!(output.stdout.contains("lease=300s"));
    assert!(output.stdout.contains("redacted=yes"));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn approval_queue_json_sample_emits_snapshot_and_done() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "approval", "queue", "--fixture", "sample"])
            .expect("parse"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::ApprovalQueueSnapshot(snapshot))
            if snapshot.pending_count == 2
                && snapshot.groups.iter().any(|group| group.resource_kind == "command")
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn approval_queue_expired_fixture_has_no_actionable_pending_items() {
    let output = render_cli_invocation(
        parse_cli_invocation(["approval", "queue", "--fixture", "expired"]).expect("parse"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Approval queue: empty"));
    assert!(output.stdout.contains("Expired: 1"));
}
