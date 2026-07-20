use std::fs;

use ruse_core::{
    ContextRetrievalQuery, ProjectContextLimits, ProjectContextRetrievalInput,
    ProjectContextVisibility, ProjectFileKind, retrieve_project_context,
};

fn limits() -> ProjectContextLimits {
    ProjectContextLimits::new(100, 8, 2_048, 20_000).expect("valid limits")
}

#[test]
fn project_context_retrieval_scans_and_selects_relevant_files() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("crates/ruse-cli/src")).expect("cli src");
    fs::create_dir_all(temp.path().join("crates/ruse-cli/tests")).expect("cli tests");
    fs::create_dir_all(temp.path().join("crates/ruse-ui-bridge/src")).expect("bridge src");
    fs::write(
        temp.path().join("crates/ruse-cli/src/command.rs"),
        "pub fn context_command() {}\n",
    )
    .expect("command");
    fs::write(
        temp.path()
            .join("crates/ruse-cli/tests/project_context_commands.rs"),
        "#[test]\nfn context_command() {}\n",
    )
    .expect("command test");
    fs::write(
        temp.path().join("crates/ruse-ui-bridge/src/frame.rs"),
        "pub struct Frame;\n",
    )
    .expect("frame");

    let report = retrieve_project_context(ProjectContextRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new("update rust CLI context command", 3, 200, Vec::new())
            .expect("valid query"),
    })
    .expect("retrieve project context");

    assert_eq!(report.summary.workspace_root_display, ".");
    let selected_paths: Vec<_> = report
        .retrieval
        .selected
        .iter()
        .map(|candidate| candidate.path.as_str())
        .collect();
    assert_eq!(
        selected_paths,
        vec![
            "crates/ruse-cli/src/command.rs",
            "crates/ruse-cli/tests/project_context_commands.rs",
            "crates/ruse-ui-bridge/src/frame.rs"
        ]
    );
    assert!(
        !serde_json::to_string(&report)
            .expect("serialize report")
            .contains(temp.path().to_str().expect("utf8 path"))
    );
}

#[test]
fn project_context_retrieval_preserves_pin_diagnostics_and_secret_metadata() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::write(temp.path().join(".env"), "api_key = \"sk-test-secret\"\n").expect("env");
    fs::write(temp.path().join("README.md"), "# Demo\n").expect("readme");

    let report = retrieve_project_context(ProjectContextRetrievalInput {
        workspace_root: temp.path().to_path_buf(),
        limits: limits(),
        query: ContextRetrievalQuery::new(
            "env secret metadata",
            2,
            20,
            vec!["missing.rs".to_string()],
        )
        .expect("valid query"),
    })
    .expect("retrieve project context");

    let env = report
        .retrieval
        .selected
        .iter()
        .find(|candidate| candidate.path == ".env")
        .expect("env selected from metadata");
    assert_eq!(env.kind, ProjectFileKind::SecretLike);
    assert_eq!(env.visibility, ProjectContextVisibility::MetadataOnly);
    assert!(report.retrieval.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "context_retrieval.pin.unknown"
            && diagnostic.path.as_deref() == Some("missing.rs")
    }));

    let serialized = serde_json::to_string(&report).expect("serialize report");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
}
