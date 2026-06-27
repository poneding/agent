use agent_cli::{
    CliCommand, CliError, ModelSelection, OutputMode, parse_cli_command, parse_cli_invocation,
    render_cli_command,
};

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
fn cli_modes_parse_prompt_profiles() {
    let command = parse_cli_command(["prompt", "profiles"]).expect("parse prompt profiles");

    assert_eq!(command, CliCommand::PromptProfiles);
}

#[test]
fn cli_modes_parse_prompt_inspect() {
    let command = parse_cli_command(["prompt", "inspect"]).expect("parse prompt inspect");

    assert_eq!(command, CliCommand::PromptInspect);
}

#[test]
fn cli_modes_parse_provider_and_model_flags() {
    let invocation = parse_cli_invocation([
        "--provider",
        "openai",
        "--model",
        "gpt-4.1-mini",
        "--prompt",
        "hello",
    ])
    .expect("parse provider and model");

    assert_eq!(
        invocation.command,
        CliCommand::BatchPrompt {
            prompt: "hello".to_string(),
        }
    );
    assert_eq!(invocation.output_mode, OutputMode::Text);
    assert_eq!(
        invocation.model_selection,
        ModelSelection {
            provider: "openai".to_string(),
            model: "gpt-4.1-mini".to_string(),
        }
    );
}

#[test]
fn cli_modes_reject_empty_provider_and_model_flags() {
    let provider_err =
        parse_cli_invocation(["--provider", "", "--prompt", "hello"]).expect_err("empty provider");
    let model_err =
        parse_cli_invocation(["--model", "", "--prompt", "hello"]).expect_err("empty model");

    assert_eq!(provider_err, CliError::EmptyModelProvider);
    assert_eq!(model_err, CliError::EmptyModelName);
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

    assert_eq!(output.stdout, "fake response: hello\n");
    assert!(output.exit);
}
