use agent_cli::{CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn final_review_parses_text_and_json_fixture_forms() {
    let text = parse_cli_invocation(["repo", "final-review", "--fixture", "local-coding"])
        .expect("parse text final review");
    assert_eq!(text.output_mode, OutputMode::Text);
    assert!(matches!(text.command, CliCommand::RepoFinalReview { .. }));

    let json = parse_cli_invocation([
        "--json",
        "repo",
        "final-review",
        "--fixture",
        "local-coding",
    ])
    .expect("parse json final review");
    assert_eq!(json.output_mode, OutputMode::Json);
    assert!(matches!(json.command, CliCommand::RepoFinalReview { .. }));
}

#[test]
fn final_review_text_fixture_includes_review_evidence() {
    let invocation = parse_cli_invocation(["repo", "final-review", "--fixture", "local-coding"])
        .expect("parse final review");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Final review"));
    assert!(output.stdout.contains("Changed files: README.md"));
    assert!(output.stdout.contains("git status --short exited 0"));
    assert!(output.stdout.contains("Audit events: 30"));
    assert!(output.stdout.contains("Replay reconstructed: yes"));
    assert!(
        output
            .stdout
            .contains("Review and commit the verified changes.")
    );
}

#[test]
fn final_review_json_fixture_emits_snapshot_and_done() {
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "final-review",
        "--fixture",
        "local-coding",
    ])
    .expect("parse final review");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::FinalReviewSnapshot(snapshot))
            if snapshot.changed_files == vec!["README.md"]
                && snapshot.audit_event_count == 30
                && snapshot.replay_reconstructed
                && snapshot.command_results[0].summary.contains("M README.md")
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
}
