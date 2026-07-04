use std::fs;

use agent_core::{
    ProjectContextLimits, ProjectContextScanError, ProjectContextScanInput,
    ProjectContextVisibility, ProjectFileKind, scan_project_context,
};

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 1_024, 10_000).expect("valid limits")
}

#[test]
fn project_context_scan_summarizes_workspace_with_stable_relative_paths() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("src")).expect("src dir");
    fs::create_dir_all(temp.path().join("tests")).expect("tests dir");
    fs::create_dir_all(temp.path().join("target/debug")).expect("target dir");
    fs::create_dir_all(temp.path().join(".git/objects")).expect("git dir");
    fs::write(temp.path().join("README.md"), "# Example\n").expect("readme");
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .expect("cargo");
    fs::write(temp.path().join("src/lib.rs"), "pub fn demo() {}\n").expect("lib");
    fs::write(
        temp.path().join("tests/integration.rs"),
        "#[test]\nfn ok() {}\n",
    )
    .expect("test");
    fs::write(temp.path().join("target/debug/build.log"), "generated\n").expect("ignored");
    fs::write(temp.path().join(".git/config"), "private\n").expect("ignored");

    let report = scan_project_context(ProjectContextScanInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
    })
    .expect("scan project context");

    let paths: Vec<_> = report
        .summary
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.kind))
        .collect();

    assert_eq!(
        paths,
        vec![
            ("Cargo.toml", ProjectFileKind::Config),
            ("README.md", ProjectFileKind::Documentation),
            ("src/lib.rs", ProjectFileKind::Source),
            ("tests/integration.rs", ProjectFileKind::Test),
        ]
    );
    assert!(
        report
            .summary
            .directories
            .iter()
            .any(|dir| dir.path == "src")
    );
    assert!(
        report
            .summary
            .omitted
            .iter()
            .any(|omission| omission.path == "target" && omission.reason == "ignored_path")
    );
    assert!(
        report
            .summary
            .omitted
            .iter()
            .any(|omission| omission.path == ".git" && omission.reason == "ignored_path")
    );
    assert_eq!(report.summary.workspace_root_display, ".");
}

#[test]
fn project_context_scan_redacts_secret_like_and_binary_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    fs::write(temp.path().join("image.bin"), b"\0\0\0binary").expect("binary");

    let report = scan_project_context(ProjectContextScanInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
    })
    .expect("scan project context");

    let env = report
        .summary
        .files
        .iter()
        .find(|file| file.path == ".env")
        .expect("env metadata");
    assert_eq!(env.kind, ProjectFileKind::SecretLike);
    assert_eq!(env.visibility, ProjectContextVisibility::MetadataOnly);

    let binary = report
        .summary
        .files
        .iter()
        .find(|file| file.path == "image.bin")
        .expect("binary metadata");
    assert_eq!(binary.kind, ProjectFileKind::Binary);
    assert_eq!(binary.visibility, ProjectContextVisibility::MetadataOnly);

    let serialized = serde_json::to_string(&report.summary).expect("serialize summary");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn project_context_scan_reports_limits_deterministically() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join("a.rs"), "fn a() {}\n").expect("a");
    fs::write(temp.path().join("b.rs"), "fn b() {}\n").expect("b");
    fs::write(temp.path().join("large.rs"), "x".repeat(64)).expect("large");

    let report = scan_project_context(ProjectContextScanInput {
        workspace_root: temp.path().to_path_buf(),
        limits: ProjectContextLimits::new(1, 8, 16, 10_000).expect("valid limits"),
    })
    .expect("scan project context");

    assert_eq!(report.summary.files.len(), 1);
    assert!(
        report
            .summary
            .omitted
            .iter()
            .any(|omission| omission.reason == "max_files")
    );
    assert!(
        report
            .summary
            .omitted
            .iter()
            .any(|omission| omission.path == "large.rs" && omission.reason == "max_file_bytes")
    );
    assert!(
        report
            .summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "project_context.limit.max_files")
    );
}

#[test]
fn project_context_scan_invalid_workspace_fails_before_summary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing");

    let error = scan_project_context(ProjectContextScanInput {
        workspace_root: missing,
        limits: limits(),
    })
    .expect_err("missing workspace rejected");

    assert!(matches!(
        error,
        ProjectContextScanError::InvalidWorkspace { .. }
    ));
    assert_eq!(error.code(), "project_context.workspace.invalid");
}
