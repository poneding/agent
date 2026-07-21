use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use ruse_eval::{
    EVAL_SCENARIO_SCHEMA_VERSION, EVAL_SUITE_SCHEMA_VERSION, EvalExpectations, EvalExpectedStatus,
    EvalFixtures, EvalPermissionProfile, EvalScenarioKind, EvalScenarioManifest, EvalStrictness,
    EvalSuiteError, EvalSuiteManifest, EvalSuiteScenarioRef, parse_eval_suite_json,
    validate_eval_suite,
};
use serde_json::json;

fn write_file(root: &Utf8Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent").as_std_path()).expect("create parent");
    fs::write(path.as_std_path(), content).expect("write file");
}

fn scenario_manifest(id: &str, fixture_path: &str) -> String {
    let manifest = EvalScenarioManifest {
        schema_version: EVAL_SCENARIO_SCHEMA_VERSION,
        id: id.to_string(),
        name: format!("{id} scenario"),
        kind: EvalScenarioKind::ReplayTrace,
        fixtures: EvalFixtures {
            replay_trace: Some(Utf8PathBuf::from(fixture_path)),
            ..EvalFixtures::default()
        },
        permissions: EvalPermissionProfile::default(),
        expectations: EvalExpectations {
            expected_status: EvalExpectedStatus::Passed,
            required_events: Vec::new(),
            context_quality: None,
            recovery: None,
            team_workflow: None,
            memory: None,
            golden: None,
        },
        metadata: Default::default(),
    };
    serde_json::to_string_pretty(&manifest).expect("scenario json")
}

fn suite_manifest() -> EvalSuiteManifest {
    EvalSuiteManifest {
        schema_version: EVAL_SUITE_SCHEMA_VERSION,
        id: "default-regression".to_string(),
        name: "Default regression".to_string(),
        default_strictness: EvalStrictness::Strict,
        scenarios: vec![
            EvalSuiteScenarioRef {
                path: Utf8PathBuf::from("scenarios/replay-one.eval.json"),
                alias: Some("replay-one".to_string()),
                labels: vec!["replay".to_string(), "smoke".to_string()],
                strictness: None,
            },
            EvalSuiteScenarioRef {
                path: Utf8PathBuf::from("scenarios/replay-two.eval.json"),
                alias: Some("replay-two".to_string()),
                labels: vec!["replay".to_string()],
                strictness: Some(EvalStrictness::Warn),
            },
        ],
    }
}

#[test]
fn suite_manifest_round_trips_and_expands_scenarios() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 root");
    write_file(
        &root,
        "scenarios/replay-one.eval.json",
        &scenario_manifest("replay-one-scenario", "fixtures/trace-one.json"),
    );
    write_file(
        &root,
        "scenarios/replay-two.eval.json",
        &scenario_manifest("replay-two-scenario", "fixtures/trace-two.json"),
    );

    let suite = suite_manifest();
    let json = serde_json::to_string_pretty(&suite).expect("serialize suite");
    let validated = parse_eval_suite_json(&json, &root).expect("valid suite");

    assert_eq!(validated.manifest.id, "default-regression");
    assert_eq!(validated.scenarios.len(), 2);
    assert_eq!(validated.scenarios[0].manifest.id, "replay-one-scenario");
    assert_eq!(
        validated.scenario_refs[0].alias.as_deref(),
        Some("replay-one")
    );
    assert_eq!(
        validated.scenario_refs[0].strictness,
        EvalStrictness::Strict
    );
    assert_eq!(validated.scenario_refs[1].strictness, EvalStrictness::Warn);
    assert_eq!(
        validated.scenario_refs[0].normalized_path,
        root.join("scenarios/replay-one.eval.json")
    );
}

#[test]
fn suite_rejects_unsupported_schema_version() {
    let mut suite = suite_manifest();
    suite.schema_version = EVAL_SUITE_SCHEMA_VERSION + 1;

    let error = validate_eval_suite(suite, "docs/eval").expect_err("schema fails");

    assert!(matches!(
        error,
        EvalSuiteError::UnsupportedSchemaVersion { .. }
    ));
}

#[test]
fn suite_rejects_invalid_id() {
    let mut suite = suite_manifest();
    suite.id = "Bad Suite".to_string();

    let error = validate_eval_suite(suite, "docs/eval").expect_err("id fails");

    assert!(matches!(
        error,
        EvalSuiteError::InvalidField { field, .. } if field == "id"
    ));
}

#[test]
fn suite_rejects_duplicate_aliases() {
    let mut suite = suite_manifest();
    suite.scenarios[1].alias = Some("replay-one".to_string());

    let error = validate_eval_suite(suite, "docs/eval").expect_err("duplicate fails");

    assert!(matches!(
        error,
        EvalSuiteError::DuplicateScenarioAlias { ref alias } if alias == "replay-one"
    ));
}

#[test]
fn suite_rejects_path_traversal() {
    let mut suite = suite_manifest();
    suite.scenarios[0].path = Utf8PathBuf::from("../outside.eval.json");

    let error = validate_eval_suite(suite, "docs/eval").expect_err("path fails");

    assert!(matches!(
        error,
        EvalSuiteError::UnsafePath { field, .. } if field == "scenarios.path"
    ));
}

#[test]
fn suite_reports_missing_scenario_file_with_context() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 root");

    let error = validate_eval_suite(suite_manifest(), &root).expect_err("missing file");

    assert!(matches!(error, EvalSuiteError::MissingScenarioFile { .. }));
    assert!(error.to_string().contains("replay-one.eval.json"));
}

#[test]
fn suite_rejects_secret_like_labels_without_echoing_values() {
    let mut suite = suite_manifest();
    suite.scenarios[0]
        .labels
        .push("api_key_sk-live-secret-value".to_string());

    let error = validate_eval_suite(suite, "docs/eval").expect_err("label fails");
    let message = error.to_string();

    assert!(matches!(
        error,
        EvalSuiteError::InvalidField { field, .. } if field == "scenarios.labels"
    ));
    assert!(!message.contains("sk-live-secret-value"));
}

#[test]
fn suite_unknown_fields_are_rejected() {
    let value = json!({
        "schema_version": EVAL_SUITE_SCHEMA_VERSION,
        "id": "default-regression",
        "name": "Default regression",
        "unexpected": true,
        "scenarios": []
    });

    let error =
        serde_json::from_value::<EvalSuiteManifest>(value).expect_err("unknown field fails");

    assert!(error.to_string().contains("unknown field"));
}
