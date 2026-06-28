use std::fs;
use std::path::{Path, PathBuf};

use xtask::{
    CliReleaseArtifact, CliReleaseManifest, PackageCliOptions, ReleaseChannel, package_cli_plan,
    sha256_file,
};

fn workspace() -> PathBuf {
    PathBuf::from("/workspace")
}

#[test]
fn package_cli_plan_uses_deterministic_artifact_paths() {
    let options = PackageCliOptions {
        channel: ReleaseChannel::Beta,
        target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
        output_dir: PathBuf::from("dist"),
    };

    let plan = package_cli_plan(
        &options,
        "aarch64-apple-darwin",
        Path::new("/workspace"),
        "1.2.3",
    );

    assert_eq!(
        plan.build_command.args,
        vec![
            "build",
            "--release",
            "-p",
            "agent-cli",
            "--bin",
            "agent",
            "--target",
            "x86_64-unknown-linux-gnu",
        ]
    );
    assert_eq!(
        plan.artifact_source,
        workspace().join("target/x86_64-unknown-linux-gnu/release/agent")
    );
    assert_eq!(
        plan.artifact_output,
        PathBuf::from("dist/agent-1.2.3-x86_64-unknown-linux-gnu-beta")
    );
    assert_eq!(
        plan.manifest_path,
        PathBuf::from("dist/agent-1.2.3-x86_64-unknown-linux-gnu-beta.json")
    );
    assert_eq!(
        plan.checksum_path,
        PathBuf::from("dist/agent-1.2.3-x86_64-unknown-linux-gnu-beta.sha256")
    );
    assert_eq!(plan.smoke_root_path, PathBuf::from("dist/smoke-root"));
    assert_eq!(
        plan.smoke_config_path,
        PathBuf::from("dist/smoke-root/config/agent-release-smoke.toml")
    );
    assert_eq!(plan.smoke_commands.len(), 4);
    assert_eq!(plan.smoke_commands[0].args, vec!["--version"]);
    assert_eq!(
        plan.smoke_commands[0].env,
        vec![
            ("HOME".to_string(), "/workspace/dist/smoke-root".to_string()),
            (
                "USERPROFILE".to_string(),
                "/workspace/dist/smoke-root".to_string(),
            ),
        ]
    );
    assert!(
        plan.smoke_commands
            .iter()
            .any(|command| command.args == vec!["--json", "--prompt", "release smoke"])
    );
}

#[test]
fn package_cli_plan_uses_host_release_binary_without_explicit_target() {
    let plan = package_cli_plan(
        &PackageCliOptions::default(),
        "aarch64-apple-darwin",
        Path::new("/workspace"),
        "1.2.3",
    );

    assert_eq!(
        plan.build_command.args,
        vec!["build", "--release", "-p", "agent-cli", "--bin", "agent"]
    );
    assert_eq!(plan.target_triple, "aarch64-apple-darwin");
    assert_eq!(
        plan.artifact_source,
        workspace().join("target/release/agent")
    );
    assert_eq!(
        plan.artifact_output,
        PathBuf::from("target/agent-release/agent-1.2.3-aarch64-apple-darwin-dev")
    );
}

#[test]
fn release_manifest_serializes_stable_metadata() {
    let manifest = CliReleaseManifest::new(
        ReleaseChannel::Stable,
        CliReleaseArtifact {
            name: "agent-1.2.3-aarch64-apple-darwin-stable".to_string(),
            target_triple: "aarch64-apple-darwin".to_string(),
            version: "1.2.3".to_string(),
            path: PathBuf::from("dist/agent-1.2.3-aarch64-apple-darwin-stable"),
            checksum_sha256: "abc123".to_string(),
        },
        vec![
            "--version".to_string(),
            "--config-check --config smoke.toml".to_string(),
        ],
    );

    let value = serde_json::to_value(&manifest).expect("serialize manifest");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["channel"], "stable");
    assert_eq!(
        value["artifact"]["name"],
        "agent-1.2.3-aarch64-apple-darwin-stable"
    );
    assert_eq!(value["artifact"]["checksum_sha256"], "abc123");
    assert_eq!(value["smoke_checks"][0], "--version");
}

#[test]
fn sha256_file_returns_hex_digest() {
    let path = std::env::temp_dir().join(format!("agent-xtask-sha256-{}.txt", std::process::id()));
    fs::write(&path, b"release artifact").expect("write fixture");

    let digest = sha256_file(&path).expect("hash file");

    assert_eq!(
        digest,
        "133cfccb5b503cf4040c95f3dfad56d07c1574283a1e39066b594f6ee33711ba"
    );
    let _ = fs::remove_file(path);
}
