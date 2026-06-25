use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .arg("--version")
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "agent-cli 0.1.0\n");
    assert!(output.stderr.is_empty());
}
