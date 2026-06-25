use std::process::Command;

use agent_cli::{
    CliCommand, CliInvocation, OutputMode, parse_cli_invocation, render_cli_invocation,
};
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid frame line"))
        .collect()
}

#[test]
fn json_mode_parse_json_local_summary() {
    let invocation =
        parse_cli_invocation(["--json", "--local-summary"]).expect("parse json local summary");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::LocalSummary,
            output_mode: OutputMode::Json,
        }
    );
}

#[test]
fn json_mode_renders_parseable_bridge_frames() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::LocalSummary,
        output_mode: OutputMode::Json,
    });

    assert!(output.exit);
    let frames = parse_frames(&output.stdout);

    assert!(matches!(frames.first(), Some(UiJsonFrame::Event(_))));
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count }) if *event_count as usize == frames.len() - 1
    ));
}

#[test]
fn json_mode_binary_prints_ndjson_frames() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .args(["--json", "--local-summary"])
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let frames = parse_frames(&stdout);
    let event_count = frames
        .iter()
        .filter(|frame| matches!(frame, UiJsonFrame::Event(_)))
        .count();

    assert!(event_count > 0);
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: done_count }) if *done_count as usize == event_count
    ));
}
