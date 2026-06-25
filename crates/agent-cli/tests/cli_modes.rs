use agent_cli::{CliCommand, CliError, parse_cli_command, render_cli_command};

#[test]
fn cli_modes_parse_version_flag() {
    let command = parse_cli_command(["--version"]).expect("parse version");

    assert_eq!(command, CliCommand::Version);
}

#[test]
fn cli_modes_parse_runtime_banner_for_no_args() {
    let command = parse_cli_command(std::iter::empty::<&str>()).expect("parse banner");

    assert_eq!(command, CliCommand::RuntimeBanner);
}

#[test]
fn cli_modes_parse_batch_prompt() {
    let command = parse_cli_command(["--prompt", "hello"]).expect("parse prompt");

    assert_eq!(
        command,
        CliCommand::BatchPrompt {
            prompt: "hello".to_string(),
        }
    );
}

#[test]
fn cli_modes_parse_tty_shell() {
    let command = parse_cli_command(["--tty"]).expect("parse tty");

    assert_eq!(command, CliCommand::TtyShell);
}

#[test]
fn cli_modes_reject_unknown_flag() {
    let error = parse_cli_command(["--wat"]).expect_err("unknown flag rejected");

    assert_eq!(error, CliError::UnknownArgument("--wat".to_string()));
}

#[test]
fn cli_modes_render_deterministic_output() {
    let output = render_cli_command(CliCommand::BatchPrompt {
        prompt: "hello".to_string(),
    });

    assert_eq!(output.stdout, "agent-cli batch prompt: hello\n");
    assert!(output.exit);
}
