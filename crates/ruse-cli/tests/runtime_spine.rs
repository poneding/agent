use std::process::Command;

use ruse_cli::{CliCommand, render_cli_command};

#[test]
fn runtime_spine_renders_runtime_banner_contract() {
    let output = render_cli_command(CliCommand::RuntimeBanner);

    assert!(output.exit);
    assert_eq!(
        output.stdout,
        "agent (ruse-model uses ruse-types, ruse-storage uses ruse-types)\n"
    );
}

#[test]
fn runtime_spine_binary_no_args_in_non_tty_prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .output()
        .expect("ruse-cli binary should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("usage: agent"));
    assert!(output.stderr.is_empty());
}
