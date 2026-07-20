use ruse_core::ReleaseChannel;
use ruse_desktop::{
    DesktopError, DesktopUpdateArtifact, DesktopUpdateDecision, UpdateManifest,
    parse_update_manifest_json, validate_update_manifest, validate_update_manifest_for_current,
};

fn valid_artifact() -> DesktopUpdateArtifact {
    DesktopUpdateArtifact {
        platform: "aarch64-apple-darwin".to_string(),
        url: "file:///tmp/ruse-desktop.app.tar.gz".to_string(),
        checksum_sha256: "a".repeat(64),
        signature: Some("local-dev-signature-placeholder".to_string()),
    }
}

fn valid_manifest() -> UpdateManifest {
    UpdateManifest {
        channel: ReleaseChannel::Dev,
        version: "0.2.0".to_string(),
        artifacts: vec![valid_artifact()],
    }
}

#[test]
fn update_manifest_accepts_valid_local_manifest() {
    let manifest = valid_manifest();

    validate_update_manifest(&manifest).expect("valid manifest");
    let decision =
        validate_update_manifest_for_current("0.1.0", manifest).expect("valid update decision");

    assert_eq!(
        decision,
        DesktopUpdateDecision::UpdateAvailable {
            current_version: "0.1.0".to_string(),
            target_version: "0.2.0".to_string(),
            artifact_count: 1,
        }
    );
}

#[test]
fn update_manifest_reports_up_to_date_without_mutation() {
    let mut manifest = valid_manifest();
    manifest.version = "0.1.0".to_string();

    let decision =
        validate_update_manifest_for_current("0.1.0", manifest).expect("valid update decision");

    assert_eq!(
        decision,
        DesktopUpdateDecision::UpToDate {
            current_version: "0.1.0".to_string(),
        }
    );
}

#[test]
fn update_manifest_rejects_invalid_checksum() {
    let mut manifest = valid_manifest();
    manifest.artifacts[0].checksum_sha256 = "not-a-checksum".to_string();

    let error = validate_update_manifest(&manifest).expect_err("invalid checksum rejected");

    assert!(matches!(error, DesktopError::InvalidUpdateManifest(_)));
}

#[test]
fn update_manifest_rejects_unsupported_channel() {
    let json = r#"{
        "channel": "nightly",
        "version": "0.2.0",
        "artifacts": [{
            "platform": "aarch64-apple-darwin",
            "url": "file:///tmp/ruse-desktop.app.tar.gz",
            "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }]
    }"#;

    let error = parse_update_manifest_json(json).expect_err("unsupported channel rejected");

    assert!(matches!(error, DesktopError::InvalidUpdateManifest(_)));
}

#[test]
fn update_manifest_rejects_missing_artifact_fields() {
    let json = r#"{
        "channel": "dev",
        "version": "0.2.0",
        "artifacts": [{
            "platform": "aarch64-apple-darwin",
            "checksum_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }]
    }"#;

    let error = parse_update_manifest_json(json).expect_err("missing url rejected");

    assert!(matches!(error, DesktopError::InvalidUpdateManifest(_)));
}

#[test]
fn update_manifest_rejects_missing_artifacts_before_update_actions() {
    let mut manifest = valid_manifest();
    manifest.artifacts.clear();

    let error =
        validate_update_manifest_for_current("0.1.0", manifest).expect_err("empty artifacts");

    assert!(matches!(error, DesktopError::InvalidUpdateManifest(_)));
}
