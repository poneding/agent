use std::process::Command;

#[test]
fn local_coding_cli_prints_fixture_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_ruse-cli"))
        .arg("--local-summary")
        .output()
        .expect("ruse-cli binary should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Local coding workflow completed. Changed files: README.md. Commands: git status --short exited 0: M README.md. Audit events: 30.\n\
Changed files:\n\
- README.md\n\
Command results:\n\
- git status --short exited 0: M README.md\n\
Audit events: 30\n"
    );
}
