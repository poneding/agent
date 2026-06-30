use agent_cli::{
    CliCommand, OutputMode, RepoSessionCommand, TerminalSessionFixture, parse_cli_invocation,
    render_cli_invocation,
};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn terminal_session_commands_parse_list_and_resume_forms() {
    let list = parse_cli_invocation(["repo", "session", "list", "--fixture", "empty"])
        .expect("parse list");
    assert_eq!(list.output_mode, OutputMode::Text);
    assert!(matches!(
        list.command,
        CliCommand::RepoSession(RepoSessionCommand::List {
            fixture: Some(TerminalSessionFixture::Empty),
            ..
        })
    ));

    let resume = parse_cli_invocation([
        "--json",
        "repo",
        "session",
        "resume",
        "--fixture",
        "sample",
        "--conversation-id",
        "00000000-0000-0000-0000-000000000592",
    ])
    .expect("parse resume");
    assert_eq!(resume.output_mode, OutputMode::Json);
    assert!(matches!(
        resume.command,
        CliCommand::RepoSession(RepoSessionCommand::Resume {
            fixture: Some(TerminalSessionFixture::Sample),
            ..
        })
    ));
}

#[test]
fn terminal_session_list_text_handles_empty_directory() {
    let invocation = parse_cli_invocation(["repo", "session", "list", "--fixture", "empty"])
        .expect("parse list");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert_eq!(output.stdout, "Terminal sessions: none\n");
}

#[test]
fn terminal_session_list_json_emits_snapshot_and_done() {
    let invocation =
        parse_cli_invocation(["--json", "repo", "session", "list", "--fixture", "sample"])
            .expect("parse list");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::TerminalSessionSnapshot(snapshot))
            if snapshot.sessions.len() == 2
                && snapshot.sessions[0].conversation_id.to_string()
                    == "00000000-0000-0000-0000-000000000592"
                && snapshot.sessions[0].status == "running_command"
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count == 0
    ));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
    assert!(!output.stdout.contains("provider-secret"));
    assert!(!output.stdout.contains("model-token"));
    assert!(!output.stdout.contains("client-secret-version"));
}

#[test]
fn terminal_session_resume_text_returns_matching_summary() {
    let list = render_cli_invocation(
        parse_cli_invocation(["repo", "session", "list", "--fixture", "sample"])
            .expect("parse list"),
    );
    let conversation_id = list
        .stdout
        .lines()
        .find_map(|line| line.strip_prefix("- "))
        .and_then(|line| line.split_whitespace().next())
        .expect("conversation id");
    let invocation = parse_cli_invocation([
        "repo",
        "session",
        "resume",
        "--fixture",
        "sample",
        "--conversation-id",
        conversation_id,
    ])
    .expect("parse resume");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Terminal session: ready"));
    assert!(output.stdout.contains(conversation_id));
    assert!(output.stdout.contains("Status: running_command"));
    assert!(!output.stdout.contains("sk-test-secret"));
    assert!(!output.stdout.contains("api_key"));
}

#[test]
fn terminal_session_resume_invalid_id_exits_two() {
    let invocation = parse_cli_invocation([
        "--json",
        "repo",
        "session",
        "resume",
        "--fixture",
        "sample",
        "--conversation-id",
        "00000000-0000-0000-0000-000000000599",
    ])
    .expect("parse resume");

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 2);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::Error { code, .. })
            if code == "terminal_session.restore.not_found"
    ));
}
