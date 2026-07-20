const CI_WORKFLOW: &str = include_str!("../../.github/workflows/ci.yml");

#[test]
fn github_actions_workflow_delegates_to_xtask_ci() {
    assert!(
        CI_WORKFLOW.contains("run: cargo xtask ci"),
        "workflow should delegate quality logic to xtask"
    );
    assert!(
        CI_WORKFLOW.contains("path: target/ruse-ci"),
        "workflow should upload quality gate evidence artifacts"
    );
    assert!(
        CI_WORKFLOW.contains("if: always()"),
        "workflow should upload artifacts even after failed gates"
    );
    assert!(
        CI_WORKFLOW.contains("actions/upload-artifact@v4"),
        "workflow should use the standard artifact uploader"
    );
    assert!(
        CI_WORKFLOW.contains("dtolnay/rust-toolchain@stable"),
        "workflow should install a stable Rust toolchain"
    );
    assert!(
        CI_WORKFLOW.contains("Swatinem/rust-cache@v2"),
        "workflow should use a Cargo-aware cache action"
    );
}

#[test]
fn github_actions_workflow_keeps_quality_commands_out_of_yaml() {
    for direct_command in [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets",
        "cargo test --workspace",
        "cargo xtask eval",
    ] {
        assert!(
            !CI_WORKFLOW.contains(direct_command),
            "workflow should not duplicate {direct_command}"
        );
    }
}
