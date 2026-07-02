use agent_cli::{
    CliCommand, CliError, OutputMode, RecoveryCliCommand, ReplayCliCommand, parse_cli_command,
    parse_cli_invocation, render_cli_invocation,
};
use agent_types::{CheckpointId, ConversationId, TurnId};
use agent_ui_bridge::UiJsonFrame;

const CONVERSATION: &str = "00000000-0000-0000-0000-000000006301";
const TURN: &str = "00000000-0000-0000-0000-000000006302";
const CHECKPOINT: &str = "00000000-0000-0000-0000-000000006303";

fn conversation_id() -> ConversationId {
    CONVERSATION.parse().expect("valid conversation id")
}

fn turn_id() -> TurnId {
    TURN.parse().expect("valid turn id")
}

fn checkpoint_id() -> CheckpointId {
    CHECKPOINT.parse().expect("valid checkpoint id")
}

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame"))
        .collect()
}

#[test]
fn recovery_and_replay_commands_parse() {
    assert_eq!(
        parse_cli_command([
            "recovery",
            "status",
            "--conversation",
            CONVERSATION,
            "--turn",
            TURN,
        ])
        .expect("parse recovery status"),
        CliCommand::Recovery(RecoveryCliCommand::Status {
            conversation_id: conversation_id(),
            turn_id: Some(turn_id()),
        })
    );

    assert_eq!(
        parse_cli_command([
            "recovery",
            "resume",
            "--conversation",
            CONVERSATION,
            "--checkpoint",
            CHECKPOINT,
            "--dry-run",
        ])
        .expect("parse recovery resume"),
        CliCommand::Recovery(RecoveryCliCommand::Resume {
            conversation_id: conversation_id(),
            turn_id: None,
            checkpoint_id: Some(checkpoint_id()),
            dry_run: true,
        })
    );

    assert_eq!(
        parse_cli_command([
            "replay",
            "inspect",
            "--conversation",
            CONVERSATION,
            "--from-checkpoint",
            CHECKPOINT,
        ])
        .expect("parse replay inspect"),
        CliCommand::Replay(ReplayCliCommand::Inspect {
            conversation_id: conversation_id(),
            from_checkpoint_id: Some(checkpoint_id()),
        })
    );

    assert_eq!(
        parse_cli_command([
            "replay",
            "fork",
            "--conversation",
            CONVERSATION,
            "--turn",
            TURN,
            "--checkpoint",
            CHECKPOINT,
            "--label",
            "follow-up",
            "--dry-run",
        ])
        .expect("parse replay fork"),
        CliCommand::Replay(ReplayCliCommand::Fork {
            conversation_id: conversation_id(),
            turn_id: Some(turn_id()),
            checkpoint_id: Some(checkpoint_id()),
            label: "follow-up".to_string(),
            dry_run: true,
        })
    );
}

#[test]
fn recovery_status_renders_text_snapshot() {
    let output = render_cli_invocation(
        parse_cli_invocation(["recovery", "status", "--conversation", CONVERSATION])
            .expect("parse invocation"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains(&format!("Recovery {CONVERSATION}")));
    assert!(output.stdout.contains("readiness=needs_approval"));
    assert!(
        output
            .stdout
            .contains("replay_from=2 replayed_event_count=4")
    );
    assert!(output.stdout.contains("pending_tool_count=1"));
    assert!(output.stdout.contains("idempotent_effect_count=1"));
    assert!(output.stdout.contains("pending_approval_count=1"));
    assert!(
        output
            .stdout
            .contains("diagnostic severity=info code=recovery.fixture")
    );
    assert!(!output.stdout.contains("must-not-appear"));
    assert!(!output.stdout.contains("sk-secret-token"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("token"));
}

#[test]
fn recovery_status_renders_json_frame() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "recovery",
            "status",
            "--conversation",
            CONVERSATION,
        ])
        .expect("parse json invocation"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert_eq!(frames.len(), 1);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::RecoverySnapshot(snapshot)
            if snapshot.snapshot.conversation_id == conversation_id()
                && snapshot.snapshot.replay.replayed_event_count == 4
                && snapshot.snapshot.pending_tool_calls.len() == 1
                && snapshot.snapshot.idempotent_effects.len() == 1
                && snapshot.snapshot.pending_approvals.len() == 1
    ));
    assert!(!output.stdout.contains("must-not-appear"));
    assert!(!output.stdout.contains("sk-secret-token"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn recovery_resume_dry_run_renders_counts() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "recovery",
            "resume",
            "--conversation",
            CONVERSATION,
            "--checkpoint",
            CHECKPOINT,
            "--dry-run",
        ])
        .expect("parse resume"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Recovery resume dry-run"));
    assert!(output.stdout.contains("blocked=no"));
    assert!(output.stdout.contains("will_replay_event_count=4"));
    assert!(output.stdout.contains("will_resume_pending_tool_count=1"));
    assert!(
        output
            .stdout
            .contains("will_reuse_idempotent_effect_count=1")
    );
    assert!(
        output
            .stdout
            .contains("diagnostic severity=info code=recovery.checkpoint.argument")
    );
}

#[test]
fn replay_inspect_and_fork_render_json_reports() {
    let inspect = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "replay",
            "inspect",
            "--conversation",
            CONVERSATION,
            "--from-checkpoint",
            CHECKPOINT,
        ])
        .expect("parse inspect"),
    );
    assert_eq!(inspect.status_code, 0);
    let frames = parse_frames(&inspect.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::ReplayInspect(report)
            if report.conversation_id == conversation_id()
                && report.replayed_event_count == 4
                && report.checkpoint_id == Some(checkpoint_id())
    ));

    let fork = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "replay",
            "fork",
            "--conversation",
            CONVERSATION,
            "--label",
            "branch sk-secret-token",
            "--dry-run",
        ])
        .expect("parse fork"),
    );
    assert_eq!(fork.status_code, 0);
    let frames = parse_frames(&fork.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::ReplayFork(report)
            if report.conversation_id == conversation_id()
                && report.dry_run
                && report.fork_label == "branch [redacted]"
    ));
    assert!(!fork.stdout.contains("sk-secret-token"));
    assert!(!fork.stdout.contains("must-not-appear"));
}

#[test]
fn replay_fork_renders_text_dry_run() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "replay",
            "fork",
            "--conversation",
            CONVERSATION,
            "--turn",
            TURN,
            "--label",
            "branch-one",
            "--dry-run",
        ])
        .expect("parse fork"),
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Replay fork dry-run"));
    assert!(output.stdout.contains("label=branch-one"));
    assert!(output.stdout.contains("replay_from=2"));
    assert!(output.stdout.contains("dry_run=yes"));
    assert!(output.stdout.contains(&format!("turn={TURN}")));
}

#[test]
fn recovery_and_replay_reject_unsafe_or_invalid_arguments() {
    assert_eq!(
        parse_cli_command(["recovery", "resume", "--conversation", CONVERSATION])
            .expect_err("missing dry run"),
        CliError::DryRunRequired("recovery resume".to_string())
    );
    assert_eq!(
        parse_cli_command([
            "replay",
            "fork",
            "--conversation",
            CONVERSATION,
            "--label",
            "branch",
        ])
        .expect_err("missing dry run"),
        CliError::DryRunRequired("replay fork".to_string())
    );
    assert_eq!(
        parse_cli_command(["recovery", "status", "--conversation", "not-a-uuid"])
            .expect_err("invalid conversation id"),
        CliError::InvalidConversationId("not-a-uuid".to_string())
    );
    assert_eq!(
        parse_cli_command([
            "replay",
            "fork",
            "--conversation",
            CONVERSATION,
            "--label",
            "",
            "--dry-run",
        ])
        .expect_err("empty label"),
        CliError::EmptyForkLabel
    );
}

#[test]
fn json_mode_invocation_preserves_output_mode() {
    let invocation = parse_cli_invocation([
        "--json",
        "recovery",
        "resume",
        "--conversation",
        CONVERSATION,
        "--dry-run",
    ])
    .expect("parse json resume");

    assert_eq!(invocation.output_mode, OutputMode::Json);
}
