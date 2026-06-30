use std::collections::BTreeMap;
use std::fs;

use agent_core::{ObservabilityEvidenceOptions, build_observability_evidence};
use agent_eval::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalEvidenceSource, EvalExpectations, EvalExpectedStatus,
    EvalFixtures, EvalPermissionProfile, EvalRunSummary, EvalRunnerOptions, EvalScenarioKind,
    EvalScenarioManifest, EvalScenarioStatus, EvalStrictness, normalized_report_golden,
    run_eval_scenario, run_eval_scenarios, validate_eval_scenario,
};
use agent_storage::load_replay_fixture_json;
use camino::{Utf8Path, Utf8PathBuf};

const M04_TRACE_V1: &str = include_str!("../../agent-storage/tests/fixtures/m04_trace_v1.json");

fn fixture_root(tempdir: &tempfile::TempDir) -> Utf8PathBuf {
    Utf8PathBuf::from_path_buf(tempdir.path().to_path_buf()).expect("utf8 tempdir")
}

fn write_fixture(root: &Utf8Path, relative: &str, content: &str) {
    let path = root.join(relative);
    let parent = path.parent().expect("fixture has parent");
    fs::create_dir_all(parent.as_std_path()).expect("create fixture dir");
    fs::write(path.as_std_path(), content).expect("write fixture");
}

fn manifest(kind: EvalScenarioKind, fixtures: EvalFixtures) -> EvalScenarioManifest {
    EvalScenarioManifest {
        schema_version: EVAL_SCENARIO_SCHEMA_VERSION,
        id: format!("scenario-{}", kind_slug(kind)),
        name: format!("{kind:?} scenario"),
        kind,
        fixtures,
        permissions: EvalPermissionProfile::default(),
        expectations: EvalExpectations {
            expected_status: EvalExpectedStatus::Passed,
            required_events: Vec::new(),
            context_quality: None,
            golden: None,
        },
        metadata: BTreeMap::new(),
    }
}

fn kind_slug(kind: EvalScenarioKind) -> &'static str {
    match kind {
        EvalScenarioKind::ReplayTrace => "replay",
        EvalScenarioKind::FakeModelTurn => "fake-model",
        EvalScenarioKind::LocalCoding => "local-coding",
        EvalScenarioKind::EvidenceOnly => "evidence",
        EvalScenarioKind::ContextQuality => "context-quality",
    }
}

#[test]
fn runner_passes_replay_trace_fixture() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "traces/m04_trace_v1.json", M04_TRACE_V1);

    let mut manifest = manifest(
        EvalScenarioKind::ReplayTrace,
        EvalFixtures {
            replay_trace: Some(Utf8PathBuf::from("traces/m04_trace_v1.json")),
            ..EvalFixtures::default()
        },
    );
    manifest.expectations.required_events = vec![
        "final_response_produced".to_string(),
        "turn_completed".to_string(),
    ];
    let scenario = validate_eval_scenario(manifest, &root).expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Passed);
    assert!(report.diagnostics.is_empty());
    let summary = report.evidence_summary.as_ref().expect("evidence summary");
    assert!(summary.event_count > 0);
    assert!(summary.trace_entries > 0);
    assert!(summary.audit_records > 0);
    assert!(summary.replayable);

    let value = serde_json::to_value(&report).expect("serialize report");
    assert_eq!(value["status"], "passed");
}

#[test]
fn runner_passes_evidence_only_fixture() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    let replay_fixture = load_replay_fixture_json(M04_TRACE_V1).expect("load replay fixture");
    let evidence = build_observability_evidence(
        &replay_fixture.events,
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )
    .expect("build evidence");
    write_fixture(
        &root,
        "evidence/m04_evidence.json",
        &serde_json::to_string_pretty(&evidence).expect("serialize evidence"),
    );

    let scenario = validate_eval_scenario(
        manifest(
            EvalScenarioKind::EvidenceOnly,
            EvalFixtures {
                evidence_bundle: Some(Utf8PathBuf::from("evidence/m04_evidence.json")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Passed);
    let summary = report.evidence_summary.expect("evidence summary");
    assert_eq!(summary.event_count, evidence.summary.total_events);
    assert_eq!(summary.audit_records, evidence.summary.audit_records);
    assert!(summary.replayable);
}

#[test]
fn runner_passes_local_coding_fixture() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "workspace/README.md", "# fixture\n");

    let mut manifest = manifest(
        EvalScenarioKind::LocalCoding,
        EvalFixtures {
            workspace: Some(Utf8PathBuf::from("workspace")),
            ..EvalFixtures::default()
        },
    );
    manifest.expectations.required_events = vec![
        "tool_call_proposed".to_string(),
        "permission_decision_recorded".to_string(),
        "tool_execution_completed".to_string(),
        "tool_observation_recorded".to_string(),
        "final_response_produced".to_string(),
    ];
    let scenario = validate_eval_scenario(manifest, &root).expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Passed);
    assert!(report.diagnostics.is_empty());
    let summary = report.evidence_summary.as_ref().expect("evidence summary");
    assert_eq!(summary.source, EvalEvidenceSource::LocalCoding);
    assert!(summary.event_count > 0);
    assert!(summary.trace_entries > 0);
    assert_eq!(summary.audit_records, 30);
    assert!(summary.replayable);
    assert_eq!(summary.diagnostics, 0);

    let value = serde_json::to_value(&report).expect("serialize report");
    assert_eq!(value["status"], "passed");
    assert_eq!(value["evidence_summary"]["source"], "local_coding");
}

#[test]
fn runner_fails_local_coding_when_required_event_is_missing() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "workspace/README.md", "# fixture\n");

    let mut manifest = manifest(
        EvalScenarioKind::LocalCoding,
        EvalFixtures {
            workspace: Some(Utf8PathBuf::from("workspace")),
            ..EvalFixtures::default()
        },
    );
    manifest.expectations.required_events = vec!["turn_completed".to_string()];
    let scenario = validate_eval_scenario(manifest, &root).expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Failed);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "required_event_missing")
    );
    let summary = report.evidence_summary.as_ref().expect("evidence summary");
    assert_eq!(summary.source, EvalEvidenceSource::LocalCoding);
    assert_eq!(summary.diagnostics, 1);
}

#[test]
fn runner_local_coding_golden_summary_is_stable_and_redacted() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "workspace/README.md", "# fixture\n");
    let scenario = validate_eval_scenario(
        manifest(
            EvalScenarioKind::LocalCoding,
            EvalFixtures {
                workspace: Some(Utf8PathBuf::from("workspace")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());
    let normalized = normalized_report_golden(&report).expect("normalized report");
    let rendered = serde_json::to_string(&normalized).expect("render normalized");

    assert!(rendered.contains("local_coding"));
    assert!(!rendered.contains(tempdir.path().to_string_lossy().as_ref()));
    assert!(!rendered.contains("sk-live-secret"));
}

#[test]
fn strict_runner_marks_unimplemented_known_kind_invalid() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "scripts/fake.json", "{}");
    let scenario = validate_eval_scenario(
        manifest(
            EvalScenarioKind::FakeModelTurn,
            EvalFixtures {
                model_script: Some(Utf8PathBuf::from("scripts/fake.json")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(
        &scenario,
        EvalRunnerOptions {
            strictness: EvalStrictness::Strict,
            ..EvalRunnerOptions::default()
        },
    );

    assert_eq!(report.status, EvalScenarioStatus::Invalid);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "scenario_kind_not_implemented")
    );
}

#[test]
fn warn_runner_skips_unimplemented_known_kind() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "scripts/fake.json", "{}");
    let scenario = validate_eval_scenario(
        manifest(
            EvalScenarioKind::FakeModelTurn,
            EvalFixtures {
                model_script: Some(Utf8PathBuf::from("scripts/fake.json")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(
        &scenario,
        EvalRunnerOptions {
            strictness: EvalStrictness::Warn,
            ..EvalRunnerOptions::default()
        },
    );

    assert_eq!(report.status, EvalScenarioStatus::Skipped);
}

#[test]
fn runner_marks_missing_fixture_read_as_failed() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    let scenario = validate_eval_scenario(
        manifest(
            EvalScenarioKind::ReplayTrace,
            EvalFixtures {
                replay_trace: Some(Utf8PathBuf::from("traces/missing.json")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid scenario");

    let report = run_eval_scenario(&scenario, EvalRunnerOptions::default());

    assert_eq!(report.status, EvalScenarioStatus::Failed);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "fixture_read_failed")
    );
}

#[test]
fn max_scenarios_marks_pending_reports_not_run() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let root = fixture_root(&tempdir);
    write_fixture(&root, "traces/m04_trace_v1.json", M04_TRACE_V1);
    let first = validate_eval_scenario(
        manifest(
            EvalScenarioKind::ReplayTrace,
            EvalFixtures {
                replay_trace: Some(Utf8PathBuf::from("traces/m04_trace_v1.json")),
                ..EvalFixtures::default()
            },
        ),
        &root,
    )
    .expect("valid first scenario");
    let mut second_manifest = manifest(
        EvalScenarioKind::ReplayTrace,
        EvalFixtures {
            replay_trace: Some(Utf8PathBuf::from("traces/m04_trace_v1.json")),
            ..EvalFixtures::default()
        },
    );
    second_manifest.id = "second-replay".to_string();
    let second = validate_eval_scenario(second_manifest, &root).expect("valid second scenario");

    let report = run_eval_scenarios(
        &[first, second],
        EvalRunnerOptions {
            max_scenarios: Some(1),
            ..EvalRunnerOptions::default()
        },
    );

    assert_eq!(
        report.summary,
        EvalRunSummary {
            total: 2,
            passed: 1,
            failed: 0,
            skipped: 0,
            invalid: 0,
            not_run: 1
        }
    );
    assert_eq!(report.scenarios[0].status, EvalScenarioStatus::Passed);
    assert_eq!(report.scenarios[1].status, EvalScenarioStatus::NotRun);
}
