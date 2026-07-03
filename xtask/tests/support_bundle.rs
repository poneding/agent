use std::fs;
use std::path::{Path, PathBuf};

use xtask::{
    ReleaseChannel, SUPPORT_BUNDLE_MANIFEST_FILE, SUPPORT_BUNDLE_SUMMARY_FILE,
    SupportBundleOptions, SupportBundleSectionStatus, build_support_bundle_manifest,
    render_support_bundle_summary, run_support_bundle_at, summarize_support_bundle,
    support_bundle_plan,
};

fn temp_workspace(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agent-support-bundle-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove stale temp workspace");
    }
    fs::create_dir_all(&path).expect("create temp workspace");
    path
}

#[test]
fn support_bundle_plan_uses_stable_paths() {
    let options = SupportBundleOptions {
        channel: ReleaseChannel::Beta,
        output_dir: PathBuf::from("dist/support"),
    };
    let plan = support_bundle_plan(&options, Path::new("/workspace"));

    assert_eq!(plan.channel, ReleaseChannel::Beta);
    assert_eq!(plan.output_dir, PathBuf::from("dist/support"));
    assert_eq!(
        plan.manifest_path,
        PathBuf::from("dist/support").join(SUPPORT_BUNDLE_MANIFEST_FILE)
    );
    assert_eq!(
        plan.summary_path,
        PathBuf::from("dist/support").join(SUPPORT_BUNDLE_SUMMARY_FILE)
    );
    assert_eq!(
        plan.release_artifact_dir,
        PathBuf::from("/workspace/target/agent-release")
    );
    assert_eq!(
        plan.eval_report_path,
        PathBuf::from("/workspace/target/agent-eval/eval-run-report.json")
    );
}

#[test]
fn support_bundle_manifest_collects_allowlisted_artifact_references() {
    let workspace = temp_workspace("manifest");
    let release_dir = workspace.join("target/agent-release");
    let ci_dir = workspace.join("target/agent-ci");
    let eval_dir = workspace.join("target/agent-eval");
    fs::create_dir_all(&release_dir).expect("release dir");
    fs::create_dir_all(&ci_dir).expect("ci dir");
    fs::create_dir_all(&eval_dir).expect("eval dir");
    fs::write(
        release_dir.join("agent-1.2.3-test-dev.json"),
        "{\"ok\":true}\n",
    )
    .expect("cli manifest");
    fs::write(
        release_dir.join("agent-desktop-1.2.3-test-dev.update.json"),
        "{\"ok\":true}\n",
    )
    .expect("desktop manifest");
    fs::write(
        release_dir.join("agent-desktop-1.2.3-test-dev.plan.txt"),
        "plan\n",
    )
    .expect("desktop plan");
    fs::write(
        ci_dir.join("quality-gate-report.json"),
        "{\"status\":\"passed\"}\n",
    )
    .expect("quality report");
    fs::write(
        eval_dir.join("eval-run-report.json"),
        "{\"status\":\"passed\"}\n",
    )
    .expect("eval report");

    let plan = support_bundle_plan(
        &SupportBundleOptions {
            channel: ReleaseChannel::Dev,
            output_dir: workspace.join("support"),
        },
        &workspace,
    );
    let manifest = build_support_bundle_manifest(&plan);

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.channel, ReleaseChannel::Dev);
    for id in [
        "release_metadata",
        "cli_release_manifest",
        "desktop_update_manifest",
        "desktop_release_plan",
        "quality_gate_report",
        "eval_report",
    ] {
        assert!(
            manifest.sections.iter().any(|section| section.id == id
                && section.status == SupportBundleSectionStatus::Included),
            "expected included section {id}"
        );
    }
    assert!(manifest.sections.iter().any(|section| {
        section.id == "onboarding_readiness"
            && section.status == SupportBundleSectionStatus::Omitted
    }));
    let serialized = serde_json::to_string(&manifest).expect("manifest json");
    assert!(!serialized.contains("sk-live-secret"));
    assert!(!serialized.contains("OPENAI_API_KEY"));
}

#[test]
fn support_bundle_writes_manifest_and_summary() {
    let workspace = temp_workspace("run");
    let summary = run_support_bundle_at(
        &SupportBundleOptions {
            channel: ReleaseChannel::Beta,
            output_dir: workspace.join("support"),
        },
        &workspace,
    )
    .expect("support bundle should run");

    assert_eq!(
        summary.manifest_path,
        workspace.join("support/support-bundle-manifest.json")
    );
    assert_eq!(
        summary.summary_path,
        workspace.join("support/support-bundle-summary.txt")
    );
    assert!(summary.manifest_path.exists());
    assert!(summary.summary_path.exists());
    let rendered = render_support_bundle_summary(&summary);
    assert!(rendered.contains("Support bundle"));
    assert!(rendered.contains("included="));
    assert!(rendered.contains("raw_secrets_omitted"));
}

#[test]
fn support_bundle_summary_counts_section_statuses() {
    let workspace = temp_workspace("summary");
    let plan = support_bundle_plan(
        &SupportBundleOptions {
            channel: ReleaseChannel::Dev,
            output_dir: workspace.join("support"),
        },
        &workspace,
    );
    let manifest = build_support_bundle_manifest(&plan);
    let summary = summarize_support_bundle(
        &manifest,
        plan.manifest_path.clone(),
        plan.summary_path.clone(),
    );

    assert!(summary.included_section_count >= 1);
    assert!(summary.omitted_section_count >= 1);
    assert_eq!(summary.failed_section_count, 0);
    assert!(
        summary
            .redaction_diagnostics
            .contains(&"raw_environment_omitted".to_string())
    );
}
