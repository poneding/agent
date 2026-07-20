use std::process::Command;

#[test]
fn prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .arg("--version")
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "agent 0.1.0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn prints_batch_prompt_output() {
    let home = tempfile::tempdir().expect("home");
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .env("HOME", home.path())
        .args(["--prompt", "hello"])
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fake response: hello\n"
    );
    assert!(output.stderr.is_empty());
}
