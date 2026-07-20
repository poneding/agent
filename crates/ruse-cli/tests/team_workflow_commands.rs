use ruse_cli::{
    CliCommand, TeamCliCommand, TeamWorkflowFixture, parse_cli_command, parse_cli_invocation,
    render_cli_invocation,
};
use ruse_team::{ReviewVote, SubagentRole, TeamWorkflowReadiness};
use ruse_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame"))
        .collect()
}

#[test]
fn team_workflow_commands_parse() {
    assert_eq!(
        parse_cli_command(["team", "status"]).expect("parse team status"),
        CliCommand::Team(TeamCliCommand::Status {
            fixture: TeamWorkflowFixture::Sample,
        })
    );

    assert_eq!(
        parse_cli_command([
            "team",
            "handoff",
            "--role",
            "reviewer",
            "--objective",
            "Review workflow output",
            "--dry-run",
        ])
        .expect("parse handoff dry-run"),
        CliCommand::Team(TeamCliCommand::HandoffDryRun {
            role: SubagentRole::Reviewer,
            objective: "Review workflow output".to_string(),
            fixture: TeamWorkflowFixture::Sample,
            dry_run: true,
        })
    );

    assert_eq!(
        parse_cli_command([
            "team",
            "blackboard",
            "inspect",
            "--fixture",
            "policy-blocked",
        ])
        .expect("parse blackboard inspect"),
        CliCommand::Team(TeamCliCommand::BlackboardInspect {
            fixture: TeamWorkflowFixture::PolicyBlocked,
        })
    );

    assert_eq!(
        parse_cli_command([
            "team",
            "review",
            "--vote",
            "request-changes",
            "--summary",
            "Needs docs",
            "--dry-run",
        ])
        .expect("parse review dry-run"),
        CliCommand::Team(TeamCliCommand::ReviewDryRun {
            vote: ReviewVote::RequestChanges,
            summary: "Needs docs".to_string(),
            fixture: TeamWorkflowFixture::Sample,
            dry_run: true,
        })
    );

    assert_eq!(
        parse_cli_command([
            "team",
            "merge-readiness",
            "--fixture",
            "conflict",
            "--dry-run",
        ])
        .expect("parse merge readiness dry-run"),
        CliCommand::Team(TeamCliCommand::MergeReadinessDryRun {
            fixture: TeamWorkflowFixture::Conflict,
            dry_run: true,
        })
    );
}

#[test]
fn team_status_renders_text_snapshot() {
    let output =
        render_cli_invocation(parse_cli_invocation(["team", "status"]).expect("parse invocation"));

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Team workflow status"));
    assert!(output.stdout.contains("status=merge_ready"));
    assert!(output.stdout.contains("readiness=merge_ready"));
    assert!(
        output
            .stdout
            .contains("capability_ceiling grants=2 policy_blocked=no")
    );
    assert!(output.stdout.contains("handoff "));
    assert!(output.stdout.contains("blackboard_entry kind=review"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("secret"));
    assert!(!output.stdout.contains("token"));
}

#[test]
fn team_status_renders_json_frame() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "team", "status"]).expect("parse invocation"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert_eq!(frames.len(), 1);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.operation == "status"
                && !frame.dry_run
                && frame.snapshot.readiness == TeamWorkflowReadiness::MergeReady
                && frame.snapshot.capability_ceiling.grant_count == 2
                && frame.snapshot.blackboard.entry_count == 2
                && frame.errors.is_empty()
    ));
}

#[test]
fn team_handoff_dry_run_uses_reduced_snapshot() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "team",
            "handoff",
            "--role",
            "researcher",
            "--objective",
            "Research M63 docs",
            "--dry-run",
        ])
        .expect("parse invocation"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.operation == "handoff_dry_run"
                && frame.dry_run
                && frame.snapshot.readiness == TeamWorkflowReadiness::WaitingOnSubagent
                && frame.snapshot.handoffs[0].role == SubagentRole::Researcher
                && frame.snapshot.handoffs[0].objective == "Research M63 docs"
    ));
}

#[test]
fn team_review_block_dry_run_blocks_merge_readiness() {
    let output = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "team",
            "review",
            "--vote",
            "block",
            "--summary",
            "Unsafe merge",
            "--dry-run",
        ])
        .expect("parse invocation"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.operation == "review_vote_dry_run"
                && frame.dry_run
                && frame.snapshot.readiness == TeamWorkflowReadiness::Blocked
                && frame.snapshot.blackboard.block_count == 1
                && frame.snapshot.merge_review.blocking_review_count == 1
    ));
}

#[test]
fn team_merge_readiness_reports_conflict_and_policy_blocked() {
    let conflict = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "team",
            "merge-readiness",
            "--fixture",
            "conflict",
            "--dry-run",
        ])
        .expect("parse invocation"),
    );
    let frames = parse_frames(&conflict.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.operation == "merge_readiness_dry_run"
                && frame.snapshot.readiness == TeamWorkflowReadiness::HasConflict
                && frame.snapshot.conflicts.len() == 1
    ));

    let policy = render_cli_invocation(
        parse_cli_invocation([
            "--json",
            "team",
            "merge-readiness",
            "--fixture",
            "policy-blocked",
            "--dry-run",
        ])
        .expect("parse invocation"),
    );
    let frames = parse_frames(&policy.stdout);
    assert!(matches!(
        &frames[0],
        UiJsonFrame::team_workflow_snapshot(frame)
            if frame.snapshot.readiness == TeamWorkflowReadiness::PolicyBlocked
                && frame.snapshot.capability_ceiling.policy_blocked
                && frame.snapshot.capability_ceiling.grant_count == 0
    ));
}

#[test]
fn team_side_effecting_commands_require_dry_run() {
    let error = parse_cli_invocation([
        "team",
        "handoff",
        "--role",
        "implementer",
        "--objective",
        "Do work",
    ])
    .expect_err("dry-run is required");
    assert!(error.to_string().contains("requires --dry-run"));

    let error = parse_cli_invocation(["team", "merge-readiness", "--fixture", "sample"])
        .expect_err("dry-run is required");
    assert!(error.to_string().contains("requires --dry-run"));
}
