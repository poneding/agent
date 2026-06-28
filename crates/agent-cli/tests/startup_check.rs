use agent_cli::{CliCommand, OutputMode, parse_cli_invocation, render_cli_invocation};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame"))
        .collect()
}

#[test]
fn startup_check_parses_text_and_json_modes() {
    let text = parse_cli_invocation(["startup-check"]).expect("parse startup check");
    assert_eq!(text.command, CliCommand::StartupCheck);
    assert_eq!(text.output_mode, OutputMode::Text);

    let json = parse_cli_invocation(["--json", "startup-check"]).expect("parse json");
    assert_eq!(json.command, CliCommand::StartupCheck);
    assert_eq!(json.output_mode, OutputMode::Json);
}

#[test]
fn startup_check_renders_text_summary() {
    let output =
        render_cli_invocation(parse_cli_invocation(["startup-check"]).expect("parse startup"));

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Startup migrations: valid"));
    assert!(output.stdout.contains("Runtime allowed: yes"));
    assert!(output.stdout.contains("Domains: 6"));
}

#[test]
fn startup_check_renders_json_snapshot() {
    let output = render_cli_invocation(
        parse_cli_invocation(["--json", "startup-check"]).expect("parse startup"),
    );

    assert_eq!(output.status_code, 0);
    let frames = parse_frames(&output.stdout);
    assert!(matches!(
        frames.first(),
        Some(UiJsonFrame::StartupMigrationSnapshot(snapshot))
            if snapshot.runtime_allowed && snapshot.domains.len() == 6
    ));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: 0 })
    ));
}
