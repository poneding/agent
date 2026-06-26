use std::process::Command;

use agent_cli::{
    CliCommand, CliInvocation, ModelSelection, OutputMode, parse_cli_invocation,
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
fn json_mode_parse_json_local_summary() {
    let invocation =
        parse_cli_invocation(["--json", "--local-summary"]).expect("parse json local summary");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::LocalSummary,
            output_mode: OutputMode::Json,
            model_selection: ModelSelection::default(),
        }
    );
}

#[test]
fn json_mode_renders_parseable_bridge_frames() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::LocalSummary,
        output_mode: OutputMode::Json,
        model_selection: ModelSelection::default(),
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
fn json_mode_renders_selected_provider_and_model() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::BatchPrompt {
            prompt: "hello".to_string(),
        },
        output_mode: OutputMode::Json,
        model_selection: ModelSelection::new("openai", "gpt-4.1-mini")
            .expect("valid model selection"),
    });

    let frames = parse_frames(&output.stdout);

    assert_eq!(
        frames.first(),
        Some(&UiJsonFrame::ModelSelection {
            provider: "openai".to_string(),
            model: "gpt-4.1-mini".to_string(),
        })
    );
    assert!(matches!(
        frames.last(),
        Some(UiJsonFrame::Done { event_count: 0 })
    ));
}

#[test]
fn json_mode_does_not_render_secret_references() {
    let output = render_cli_invocation(CliInvocation {
        command: CliCommand::BatchPrompt {
            prompt: "hello".to_string(),
        },
        output_mode: OutputMode::Json,
        model_selection: ModelSelection::new("openai", "gpt-4.1-mini")
            .expect("valid model selection"),
    });

    assert!(!output.stdout.contains("OPENAI_API_KEY"));
    assert!(!output.stdout.contains("secret"));
    assert!(!output.stdout.contains("sk-"));
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
