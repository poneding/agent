use std::process::Command;

#[test]
fn runtime_spine_prints_runtime_banner() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-cli"))
        .output()
        .expect("agent-cli binary should run");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "agent (agent-model uses agent-types, agent-storage uses agent-types)\n"
    );
    assert!(output.stderr.is_empty());
}
