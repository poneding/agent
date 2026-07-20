use std::process::Command;

#[test]
fn trace_commands_cli_prints_local_trace_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .arg("--trace-summary")
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 tool_call proposed workspace.read_file call-read-readme"));
    assert!(stdout.contains("2 permission allow: fixture policy allows tool"));
    assert!(stdout.contains("3 tool_start started workspace.read_file call-read-readme"));
    assert!(stdout.contains("30 tool_observation observed call-git-diff"));
    assert!(stdout.contains("31 final final response: Local coding workflow completed."));
}
