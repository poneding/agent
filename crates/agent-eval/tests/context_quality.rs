use std::collections::BTreeMap;
use std::fs;

use agent_eval::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalContextQualityExpectations, EvalEvidenceSource,
    EvalExpectations, EvalExpectedStatus, EvalFixtures, EvalPermissionProfile, EvalRunnerOptions,
    EvalScenarioKind, EvalScenarioManifest, EvalScenarioStatus, run_eval_scenario,
    validate_eval_scenario,
};
use camino::{Utf8Path, Utf8PathBuf};

fn write_file(root: &Utf8Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture parent").as_std_path())
        .expect("create fixture parent");
    fs::write(path.as_std_path(), content).expect("write fixture file");
}

fn fixture_workspace(root: &Utf8Path) {
    write_file(root, "workspace/README.md", "# Demo\n");
    write_file(
        root,
        "workspace/src/command.rs",
        "pub fn command_palette() {}\n",
    );
    write_file(root, "workspace/src/http.rs", "pub fn request() {}\n");
    write_file(
        root,
        "workspace/docs/design.md",
        &format!("# Design\n{}\n", "command palette design ".repeat(32)),
    );
}

fn manifest(expectations: EvalContextQualityExpectations) -> EvalScenarioManifest {
    EvalScenarioManifest {
        schema_version: EVAL_SCENARIO_SCHEMA_VERSION,
        id: "context-quality".to_string(),
        name: "Context quality".to_string(),
        kind: EvalScenarioKind::ContextQuality,
        fixtures: EvalFixtures {
            workspace: Some(Utf8PathBuf::from("workspace")),
            ..EvalFixtures::default()
        },
        permissions: EvalPermissionProfile::default(),
        expectations: EvalExpectations {
            expected_status: EvalExpectedStatus::Passed,
            required_events: Vec::new(),
            context_quality: Some(expectations),
            recovery: None,
            team_workflow: None,
            memory: None,
            golden: None,
        },
        metadata: BTreeMap::new(),
    }
}

#[test]
fn runner_passes_context_quality_fixture() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 fixture root");
    fixture_workspace(&root);

    let scenario = validate_eval_scenario(
        manifest(EvalContextQualityExpectations {
            task: "command palette".to_string(),
            required_selected_paths: vec!["src/command.rs".to_string()],
            required_pinned_paths: vec!["README.md".to_string()],
            max_diagnostics: Some(0),
            require_retrieval_included: true,
        }),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Passed);
    assert!(report.diagnostics.is_empty());
    let summary = report.evidence_summary.expect("evidence summary");
    assert_eq!(summary.source, EvalEvidenceSource::ContextQuality);
    assert!(summary.trace_entries > 0);
    assert!(summary.replayable);
    assert_eq!(summary.diagnostics, 0);
}

#[test]
fn runner_fails_context_quality_when_required_selection_is_missing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 fixture root");
    fixture_workspace(&root);

    let scenario = validate_eval_scenario(
        manifest(EvalContextQualityExpectations {
            task: "command palette".to_string(),
            required_selected_paths: vec!["src/missing.rs".to_string()],
            required_pinned_paths: Vec::new(),
            max_diagnostics: Some(0),
            require_retrieval_included: false,
        }),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Failed);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "context_quality_selected_path_missing"
            && diagnostic.field.as_deref()
                == Some("expectations.context_quality.required_selected_paths")
    }));
}
