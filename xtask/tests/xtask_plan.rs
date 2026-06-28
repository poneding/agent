use std::path::PathBuf;

use xtask::{
    CargoCommand, PackageCliOptions, ReleaseChannel, Task, XtaskError, command_plan_for, help_text,
    parse_args,
};

fn cargo(args: &[&str]) -> CargoCommand {
    CargoCommand {
        program: "cargo".to_string(),
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
    }
}

#[test]
fn no_args_and_help_print_the_command_list() {
    assert_eq!(parse_args(Vec::<&str>::new()), Ok(Task::Help));
    assert_eq!(parse_args(["help"]), Ok(Task::Help));

    let help = help_text();
    for command in [
        "check",
        "fmt",
        "fmt-check",
        "clippy",
        "test",
        "cli-prompt",
        "cli-json",
        "package-cli",
        "desktop-test",
    ] {
        assert!(help.contains(command), "help text should mention {command}");
    }
}

#[test]
fn standard_tasks_map_to_single_cargo_commands() {
    assert_eq!(command_plan_for(&Task::Fmt), vec![cargo(&["fmt", "--all"])]);
    assert_eq!(
        command_plan_for(&Task::FmtCheck),
        vec![cargo(&["fmt", "--all", "--", "--check"])]
    );
    assert_eq!(
        command_plan_for(&Task::Clippy),
        vec![cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ])]
    );
    assert_eq!(
        command_plan_for(&Task::Test),
        vec![cargo(&["test", "--workspace"])]
    );
    assert_eq!(
        command_plan_for(&Task::DesktopTest),
        vec![cargo(&["test", "-p", "agent-desktop"])]
    );
}

#[test]
fn check_runs_the_standard_validation_commands_in_order() {
    assert_eq!(
        command_plan_for(&Task::Check),
        vec![
            cargo(&["fmt", "--all", "--", "--check"]),
            cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]),
            cargo(&["test", "--workspace"]),
        ]
    );
}

#[test]
fn cli_tasks_join_prompt_words_and_forward_to_agent_cli() {
    assert_eq!(
        parse_args(["cli-prompt", "--", "hello", "agent"]),
        Ok(Task::CliPrompt {
            prompt: "hello agent".to_string(),
        })
    );
    assert_eq!(
        command_plan_for(&Task::CliPrompt {
            prompt: "hello agent".to_string(),
        }),
        vec![cargo(&[
            "run",
            "-p",
            "agent-cli",
            "--",
            "--prompt",
            "hello agent",
        ])]
    );

    assert_eq!(
        parse_args(["cli-json", "--", "hello", "json"]),
        Ok(Task::CliJson {
            prompt: "hello json".to_string(),
        })
    );
    assert_eq!(
        command_plan_for(&Task::CliJson {
            prompt: "hello json".to_string(),
        }),
        vec![cargo(&[
            "run",
            "-p",
            "agent-cli",
            "--",
            "--json",
            "--prompt",
            "hello json",
        ])]
    );
}

#[test]
fn cli_tasks_reject_missing_prompts() {
    assert_eq!(
        parse_args(["cli-prompt", "--"]),
        Err(XtaskError::MissingPrompt("cli-prompt".to_string()))
    );
    assert_eq!(
        parse_args(["cli-json"]),
        Err(XtaskError::MissingPrompt("cli-json".to_string()))
    );
}

#[test]
fn unknown_tasks_are_rejected() {
    assert_eq!(
        parse_args(["ship-it"]),
        Err(XtaskError::UnknownCommand("ship-it".to_string()))
    );
}

#[test]
fn package_cli_parses_release_arguments() {
    assert_eq!(
        parse_args([
            "package-cli",
            "--channel",
            "beta",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--out-dir",
            "target/dist"
        ]),
        Ok(Task::PackageCli(PackageCliOptions {
            channel: ReleaseChannel::Beta,
            target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
            output_dir: PathBuf::from("target/dist"),
        }))
    );

    assert_eq!(
        parse_args(["package-cli"]),
        Ok(Task::PackageCli(PackageCliOptions::default()))
    );
}

#[test]
fn package_cli_rejects_invalid_arguments() {
    assert_eq!(
        parse_args(["package-cli", "--channel", "nightly"]),
        Err(XtaskError::InvalidReleaseChannel("nightly".to_string()))
    );
    assert_eq!(
        parse_args(["package-cli", "--target"]),
        Err(XtaskError::MissingValue("--target".to_string()))
    );
    assert_eq!(
        parse_args(["package-cli", "--wat"]),
        Err(XtaskError::UnknownOption("--wat".to_string()))
    );
}
