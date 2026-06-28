use std::fs;
use std::path::{Path, PathBuf};

use xtask::{PackageDesktopOptions, ReleaseChannel, package_desktop_plan};

fn workspace() -> PathBuf {
    PathBuf::from("/workspace")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn package_desktop_plan_uses_deterministic_paths_and_smoke_checks() {
    let options = PackageDesktopOptions {
        channel: ReleaseChannel::Beta,
        target_triple: Some("aarch64-apple-darwin".to_string()),
        output_dir: PathBuf::from("dist/desktop"),
        bundle: false,
    };

    let plan = package_desktop_plan(
        &options,
        "x86_64-unknown-linux-gnu",
        Path::new("/workspace"),
        "1.2.3",
    );

    assert_eq!(plan.target_triple, "aarch64-apple-darwin");
    assert_eq!(
        plan.bundle_command.args,
        vec![
            "--prefix",
            "apps/desktop",
            "run",
            "tauri",
            "--",
            "build",
            "--target",
            "aarch64-apple-darwin",
        ]
    );
    assert_eq!(
        plan.plan_artifact_path,
        PathBuf::from("dist/desktop/agent-desktop-1.2.3-aarch64-apple-darwin-beta.plan.txt")
    );
    assert_eq!(
        plan.update_manifest_path,
        PathBuf::from("dist/desktop/agent-desktop-1.2.3-aarch64-apple-darwin-beta.update.json")
    );
    assert_eq!(
        plan.tauri_config_path,
        workspace().join("apps/desktop/src-tauri/tauri.conf.json")
    );
    assert_eq!(
        plan.smoke_root_path,
        PathBuf::from("dist/desktop/desktop-smoke-root")
    );
    assert_eq!(
        plan.smoke_commands[0].env,
        vec![
            (
                "HOME".to_string(),
                "/workspace/dist/desktop/desktop-smoke-root".to_string(),
            ),
            (
                "USERPROFILE".to_string(),
                "/workspace/dist/desktop/desktop-smoke-root".to_string(),
            ),
        ]
    );
    assert!(plan.smoke_commands.iter().any(|command| {
        command.program == "cargo"
            && command.args == vec!["test", "-p", "agent-desktop", "update_manifest"]
    }));
    assert!(!plan.bundle);
}

#[test]
fn package_desktop_plan_can_request_real_bundle_build() {
    let options = PackageDesktopOptions {
        bundle: true,
        ..PackageDesktopOptions::default()
    };

    let plan = package_desktop_plan(
        &options,
        "aarch64-apple-darwin",
        Path::new("/workspace"),
        "1.2.3",
    );

    assert!(plan.bundle);
    assert_eq!(
        plan.bundle_command.args,
        vec!["--prefix", "apps/desktop", "run", "tauri", "--", "build",]
    );
}

#[test]
fn tauri_bundle_config_enables_local_release_metadata() {
    let config = fs::read_to_string(repo_root().join("apps/desktop/src-tauri/tauri.conf.json"))
        .expect("tauri config should be readable");
    let value: serde_json::Value = serde_json::from_str(&config).expect("valid tauri config json");

    assert_eq!(value["productName"], "agent");
    assert_eq!(value["identifier"], "run.agent.desktop");
    assert_eq!(value["bundle"]["active"], true);
    assert_eq!(value["bundle"]["targets"], "all");
    assert_eq!(value["bundle"]["icon"][0], "icons/icon.png");
}
