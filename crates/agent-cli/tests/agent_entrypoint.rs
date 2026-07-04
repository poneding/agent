use std::process::Command;

use agent_cli::{
    CliCommand, CliInvocation, InteractiveMode, ModelSelection, OutputMode, StdioProfile,
    parse_cli_command, parse_cli_invocation, render_cli_invocation_with_stdio,
};
use agent_core::ConfigOverride;
use agent_ui_bridge::UiJsonFrame;

fn parse_frames(output: &str) -> Vec<UiJsonFrame> {
    output
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid json frame line"))
        .collect()
}

#[test]
fn agent_entrypoint_parses_tui_and_plain_repl_commands() {
    assert_eq!(
        parse_cli_command(["tui"]).expect("parse tui"),
        CliCommand::TuiShell
    );
    assert_eq!(
        parse_cli_command(["repl", "--plain"]).expect("parse plain repl"),
        CliCommand::PlainRepl
    );
}

#[test]
fn agent_entrypoint_selects_tui_for_interactive_no_args() {
    let invocation = parse_cli_invocation(std::iter::empty::<&str>()).expect("parse no args");
    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: true,
            stdout_tty: true,
        },
    );

    assert!(!output.exit);
    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("agent tui"));
}

#[test]
fn agent_entrypoint_fails_non_tty_no_args_with_help() {
    let invocation = parse_cli_invocation(std::iter::empty::<&str>()).expect("parse no args");
    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: false,
            stdout_tty: false,
        },
    );

    assert!(output.exit);
    assert_ne!(output.status_code, 0);
    assert!(output.stdout.contains("usage: agent"));
    assert!(output.stdout.contains("agent --prompt"));
}

#[test]
fn agent_entrypoint_preserves_json_prompt_script_mode() {
    let invocation =
        parse_cli_invocation(["--json", "--prompt", "hello"]).expect("parse json prompt");

    assert_eq!(
        invocation,
        CliInvocation {
            command: CliCommand::BatchPrompt {
                prompt: "hello".to_string(),
            },
            output_mode: OutputMode::Json,
            model_selection: ModelSelection::default(),
            config_path: None,
            config_override: ConfigOverride::default(),
        }
    );

    let output = render_cli_invocation_with_stdio(
        invocation,
        StdioProfile {
            stdin_tty: true,
            stdout_tty: true,
        },
    );
    let frames = parse_frames(&output.stdout);

    assert!(output.exit);
    assert_eq!(output.status_code, 0);
    assert!(!output.stdout.contains("agent tui"));
    assert!(matches!(frames.first(), Some(UiJsonFrame::Event(_))));
    assert!(matches!(frames.last(), Some(UiJsonFrame::Done { .. })));
}

#[test]
fn agent_binary_runs_version_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent"))
        .arg("--version")
        .output()
        .expect("agent binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("agent "));
}

#[test]
fn legacy_agent_cli_binary_still_runs_version_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .arg("--version")
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn interactive_mode_contract_is_public_and_stable() {
    assert_eq!(InteractiveMode::Auto, InteractiveMode::Auto);
    assert_eq!(InteractiveMode::Tui, InteractiveMode::Tui);
    assert_eq!(InteractiveMode::Plain, InteractiveMode::Plain);
}
