use std::collections::BTreeMap;

use agent_eval::{
    EVAL_SCENARIO_SCHEMA_VERSION, EvalExpectations, EvalExpectedStatus, EvalFixtures,
    EvalManifestError, EvalMemoryExpectations, EvalPermissionProfile, EvalScenarioKind,
    EvalScenarioManifest, EvalTeamWorkflowExpectations, validate_eval_scenario,
};
use camino::Utf8PathBuf;
use serde_json::json;

fn replay_manifest() -> EvalScenarioManifest {
    EvalScenarioManifest {
        schema_version: EVAL_SCENARIO_SCHEMA_VERSION,
        id: "m04-replay-smoke".to_string(),
        name: "M04 replay smoke".to_string(),
        kind: EvalScenarioKind::ReplayTrace,
        fixtures: EvalFixtures {
            replay_trace: Some(Utf8PathBuf::from("traces/m04_trace_v1.json")),
            ..EvalFixtures::default()
        },
        permissions: EvalPermissionProfile::default(),
        expectations: EvalExpectations {
            expected_status: EvalExpectedStatus::Passed,
            required_events: vec!["turn_started".to_string(), "turn_completed".to_string()],
            context_quality: None,
            recovery: None,
            team_workflow: None,
            memory: None,
            golden: Some(Utf8PathBuf::from("goldens/m04_replay.json")),
        },
        metadata: BTreeMap::from([("owner".to_string(), "runtime".to_string())]),
    }
}

#[test]
fn manifest_round_trips_and_validates_replay_trace() {
    let manifest = replay_manifest();

    let json = serde_json::to_string_pretty(&manifest).expect("serialize manifest");
    let parsed: EvalScenarioManifest = serde_json::from_str(&json).expect("deserialize manifest");

    let fixture_root = Utf8PathBuf::from("fixtures/eval");
    let validated = validate_eval_scenario(parsed, &fixture_root).expect("valid replay manifest");

    assert_eq!(validated.manifest.id, "m04-replay-smoke");
    assert_eq!(validated.normalized_fixture_root, fixture_root);
    assert_eq!(
        validated
            .normalized_fixtures
            .replay_trace
            .as_ref()
            .map(|path| path.as_str()),
        Some("fixtures/eval/traces/m04_trace_v1.json")
    );
    assert!(validated.diagnostics.is_empty());
}

#[test]
fn unknown_expectation_fields_are_rejected() {
    let value = json!({
        "schema_version": EVAL_SCENARIO_SCHEMA_VERSION,
        "id": "unknown-field",
        "name": "Unknown field",
        "kind": "replay_trace",
        "fixtures": {
            "replay_trace": "trace.json"
        },
        "permissions": {},
        "expectations": {
            "expected_status": "passed",
            "unexpected": true
        }
    });

    let error =
        serde_json::from_value::<EvalScenarioManifest>(value).expect_err("unknown field fails");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn invalid_schema_version_fails_before_execution() {
    let mut manifest = replay_manifest();
    manifest.schema_version = EVAL_SCENARIO_SCHEMA_VERSION + 1;

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("schema fails");

    assert!(matches!(
        error,
        EvalManifestError::UnsupportedSchemaVersion { .. }
    ));
}

#[test]
fn invalid_scenario_id_fails_validation() {
    let mut manifest = replay_manifest();
    manifest.id = "Not A Stable Slug".to_string();

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("id fails");

    assert!(matches!(
        error,
        EvalManifestError::InvalidField { field, .. } if field == "id"
    ));
}

#[test]
fn path_traversal_fails_validation() {
    let mut manifest = replay_manifest();
    manifest.fixtures.replay_trace = Some(Utf8PathBuf::from("../secret/trace.json"));

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("path fails");

    assert!(matches!(
        error,
        EvalManifestError::UnsafePath { field, .. } if field == "fixtures.replay_trace"
    ));
    assert!(error.to_string().contains("path traversal"));
}

#[test]
fn absolute_paths_outside_fixture_root_fail_validation() {
    let mut manifest = replay_manifest();
    manifest.fixtures.replay_trace = Some(Utf8PathBuf::from("/tmp/other/trace.json"));

    let error = validate_eval_scenario(manifest, "/tmp/eval-fixtures").expect_err("path fails");

    assert!(matches!(
        error,
        EvalManifestError::UnsafePath { field, .. } if field == "fixtures.replay_trace"
    ));
    assert!(error.to_string().contains("outside the fixture root"));
}

#[test]
fn kind_required_fixture_is_enforced() {
    let mut manifest = replay_manifest();
    manifest.kind = EvalScenarioKind::EvidenceOnly;
    manifest.fixtures = EvalFixtures::default();

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("fixture fails");

    assert!(matches!(
        error,
        EvalManifestError::MissingFixture { field, .. } if field == "fixtures.evidence_bundle"
    ));
}

#[test]
fn team_workflow_smoke_manifest_validates_without_fixtures() {
    let mut manifest = replay_manifest();
    manifest.id = "m63-team-workflow-smoke".to_string();
    manifest.name = "M63 team workflow smoke".to_string();
    manifest.kind = EvalScenarioKind::TeamWorkflowSmoke;
    manifest.fixtures = EvalFixtures::default();
    manifest.expectations.required_events = Vec::new();
    manifest.expectations.team_workflow = Some(EvalTeamWorkflowExpectations {
        min_worktree_count: Some(1),
        min_handoff_count: Some(1),
        min_blackboard_entry_count: Some(1),
        min_conflict_count: Some(1),
        min_merge_blocker_count: Some(1),
        require_capability_reduction: true,
        require_policy_blocked: true,
        require_conflict_blocker: true,
        require_review_blocker: true,
        require_redaction: true,
    });

    let validated =
        validate_eval_scenario(manifest, "fixtures/eval").expect("valid team workflow manifest");

    assert_eq!(validated.manifest.kind, EvalScenarioKind::TeamWorkflowSmoke);
    assert!(validated.normalized_fixtures.replay_trace.is_none());
    assert!(validated.normalized_fixtures.workspace.is_none());
}

#[test]
fn team_workflow_smoke_requires_expectations() {
    let mut manifest = replay_manifest();
    manifest.id = "m63-team-workflow-smoke".to_string();
    manifest.kind = EvalScenarioKind::TeamWorkflowSmoke;
    manifest.fixtures = EvalFixtures::default();
    manifest.expectations.team_workflow = None;

    let error =
        validate_eval_scenario(manifest, "fixtures/eval").expect_err("expectations required");

    assert!(matches!(
        error,
        EvalManifestError::InvalidField { field, .. } if field == "expectations.team_workflow"
    ));
}

#[test]
fn memory_smoke_manifest_validates_without_fixtures() {
    let mut manifest = replay_manifest();
    manifest.id = "m64-memory-smoke".to_string();
    manifest.name = "M64 memory smoke".to_string();
    manifest.kind = EvalScenarioKind::MemorySmoke;
    manifest.fixtures = EvalFixtures::default();
    manifest.expectations.required_events = Vec::new();
    manifest.expectations.memory = Some(EvalMemoryExpectations {
        min_record_count: Some(2),
        min_proposal_count: Some(2),
        min_selected_count: Some(1),
        min_tombstone_count: Some(1),
        min_import_accepted_count: Some(1),
        min_rebuild_stale_removed_count: Some(1),
        require_proposal_decisions: true,
        require_import_export: true,
        require_rebuild: true,
        require_disabled_diagnostic: true,
        require_redaction: true,
    });

    let validated =
        validate_eval_scenario(manifest, "fixtures/eval").expect("valid memory smoke manifest");

    assert_eq!(validated.manifest.kind, EvalScenarioKind::MemorySmoke);
    assert!(validated.normalized_fixtures.replay_trace.is_none());
    assert!(validated.normalized_fixtures.workspace.is_none());
}

#[test]
fn memory_smoke_requires_expectations() {
    let mut manifest = replay_manifest();
    manifest.id = "m64-memory-smoke".to_string();
    manifest.kind = EvalScenarioKind::MemorySmoke;
    manifest.fixtures = EvalFixtures::default();
    manifest.expectations.memory = None;

    let error =
        validate_eval_scenario(manifest, "fixtures/eval").expect_err("expectations required");

    assert!(matches!(
        error,
        EvalManifestError::InvalidField { field, .. } if field == "expectations.memory"
    ));
}

#[test]
fn permissions_default_to_read_only_and_offline() {
    let permissions = EvalPermissionProfile::default();

    assert!(!permissions.allow_network);
    assert!(!permissions.allow_writes);
    assert!(permissions.allowed_commands.is_empty());
    assert!(permissions.env.is_empty());
}

#[test]
fn env_permissions_are_keys_not_values() {
    let mut manifest = replay_manifest();
    manifest.permissions.env = vec!["OPENAI_API_KEY=sk-live-secret-value".to_string()];

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("env fails");
    let message = error.to_string();

    assert!(matches!(
        error,
        EvalManifestError::InvalidField { field, .. } if field == "permissions.env"
    ));
    assert!(!message.contains("sk-live-secret-value"));
}

#[test]
fn secret_like_metadata_keys_do_not_echo_values() {
    let mut manifest = replay_manifest();
    manifest
        .metadata
        .insert("api_key".to_string(), "sk-live-secret-value".to_string());

    let error = validate_eval_scenario(manifest, "fixtures/eval").expect_err("metadata fails");
    let message = error.to_string();

    assert!(matches!(
        error,
        EvalManifestError::SecretLikeMetadataKey { ref key } if key == "api_key"
    ));
    assert!(!message.contains("sk-live-secret-value"));
}
