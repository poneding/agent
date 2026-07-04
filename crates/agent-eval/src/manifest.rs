use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const EVAL_SCENARIO_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvalScenarioManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub kind: EvalScenarioKind,
    pub fixtures: EvalFixtures,
    #[serde(default)]
    pub permissions: EvalPermissionProfile,
    pub expectations: EvalExpectations,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalScenarioKind {
    ReplayTrace,
    FakeModelTurn,
    LocalCoding,
    EvidenceOnly,
    ContextQuality,
    RecoveryCrashSmoke,
    TeamWorkflowSmoke,
    MemorySmoke,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalFixtures {
    pub replay_trace: Option<Utf8PathBuf>,
    pub evidence_bundle: Option<Utf8PathBuf>,
    pub model_script: Option<Utf8PathBuf>,
    pub workspace: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalPermissionProfile {
    pub allow_network: bool,
    pub allow_writes: bool,
    pub allowed_commands: Vec<String>,
    pub env: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvalExpectations {
    pub expected_status: EvalExpectedStatus,
    #[serde(default)]
    pub required_events: Vec<String>,
    #[serde(default)]
    pub context_quality: Option<EvalContextQualityExpectations>,
    #[serde(default)]
    pub recovery: Option<EvalRecoveryExpectations>,
    #[serde(default)]
    pub team_workflow: Option<EvalTeamWorkflowExpectations>,
    #[serde(default)]
    pub memory: Option<EvalMemoryExpectations>,
    #[serde(default)]
    pub golden: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalContextQualityExpectations {
    pub task: String,
    pub required_selected_paths: Vec<String>,
    pub required_pinned_paths: Vec<String>,
    pub max_diagnostics: Option<usize>,
    pub require_retrieval_included: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalRecoveryExpectations {
    pub expected_readiness: Option<String>,
    pub min_replayed_event_count: Option<u32>,
    pub min_idempotent_effect_count: Option<u32>,
    pub min_pending_tool_count: Option<u32>,
    pub min_conflict_count: Option<u32>,
    pub requires_retry_plan: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalTeamWorkflowExpectations {
    pub min_worktree_count: Option<u32>,
    pub min_handoff_count: Option<u32>,
    pub min_blackboard_entry_count: Option<u32>,
    pub min_conflict_count: Option<u32>,
    pub min_merge_blocker_count: Option<u32>,
    pub require_capability_reduction: bool,
    pub require_policy_blocked: bool,
    pub require_conflict_blocker: bool,
    pub require_review_blocker: bool,
    pub require_redaction: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct EvalMemoryExpectations {
    pub min_record_count: Option<u32>,
    pub min_proposal_count: Option<u32>,
    pub min_selected_count: Option<u32>,
    pub min_tombstone_count: Option<u32>,
    pub min_import_accepted_count: Option<u32>,
    pub min_rebuild_stale_removed_count: Option<u32>,
    pub require_proposal_decisions: bool,
    pub require_import_export: bool,
    pub require_rebuild: bool,
    pub require_disabled_diagnostic: bool,
    pub require_redaction: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalExpectedStatus {
    Passed,
    Failed,
    Skipped,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedEvalScenario {
    pub manifest: EvalScenarioManifest,
    pub normalized_fixture_root: Utf8PathBuf,
    pub normalized_fixtures: ValidatedEvalFixtures,
    pub diagnostics: Vec<EvalDiagnostic>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidatedEvalFixtures {
    pub replay_trace: Option<Utf8PathBuf>,
    pub evidence_bundle: Option<Utf8PathBuf>,
    pub model_script: Option<Utf8PathBuf>,
    pub workspace: Option<Utf8PathBuf>,
    pub golden: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvalDiagnostic {
    pub severity: EvalDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EvalManifestError {
    #[error("failed to parse eval scenario manifest: {0}")]
    Deserialize(String),
    #[error("unsupported eval scenario schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { supported: u32, found: u32 },
    #[error("invalid eval scenario field {field}: {message}")]
    InvalidField {
        field: &'static str,
        message: String,
    },
    #[error("missing fixture for {kind:?}: {field}")]
    MissingFixture {
        kind: EvalScenarioKind,
        field: &'static str,
    },
    #[error("unsafe path in {field}: {message}")]
    UnsafePath {
        field: &'static str,
        path: Utf8PathBuf,
        message: String,
    },
    #[error("metadata key {key} looks secret-like and cannot be stored in eval manifests")]
    SecretLikeMetadataKey { key: String },
}

pub fn parse_eval_scenario_json<P>(
    input: &str,
    fixture_root: P,
) -> Result<ValidatedEvalScenario, EvalManifestError>
where
    P: AsRef<Path>,
{
    let manifest = serde_json::from_str(input)
        .map_err(|error| EvalManifestError::Deserialize(error.to_string()))?;
    validate_eval_scenario(manifest, fixture_root)
}

pub fn validate_eval_scenario<P>(
    manifest: EvalScenarioManifest,
    fixture_root: P,
) -> Result<ValidatedEvalScenario, EvalManifestError>
where
    P: AsRef<Path>,
{
    if manifest.schema_version != EVAL_SCENARIO_SCHEMA_VERSION {
        return Err(EvalManifestError::UnsupportedSchemaVersion {
            supported: EVAL_SCENARIO_SCHEMA_VERSION,
            found: manifest.schema_version,
        });
    }

    validate_scenario_id(&manifest.id)?;
    validate_non_empty("name", &manifest.name)?;
    validate_permissions(&manifest.permissions)?;
    validate_context_quality_expectations(manifest.kind, &manifest.expectations)?;
    validate_recovery_expectations(manifest.kind, &manifest.expectations)?;
    validate_team_workflow_expectations(manifest.kind, &manifest.expectations)?;
    validate_memory_expectations(manifest.kind, &manifest.expectations)?;
    validate_metadata(&manifest.metadata)?;

    let normalized_fixture_root = normalize_root(fixture_root)?;
    let normalized_fixtures = normalize_fixtures(&manifest, &normalized_fixture_root)?;

    Ok(ValidatedEvalScenario {
        manifest,
        normalized_fixture_root,
        normalized_fixtures,
        diagnostics: Vec::new(),
    })
}

fn validate_scenario_id(id: &str) -> Result<(), EvalManifestError> {
    validate_non_empty("id", id)?;

    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err(EvalManifestError::InvalidField {
            field: "id",
            message: "must not be empty".to_string(),
        });
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(EvalManifestError::InvalidField {
            field: "id",
            message: "must start with a lowercase letter or digit".to_string(),
        });
    }

    if !id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(EvalManifestError::InvalidField {
            field: "id",
            message: "must contain only lowercase letters, digits, '-' or '_'".to_string(),
        });
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EvalManifestError> {
    if value.trim().is_empty() {
        return Err(EvalManifestError::InvalidField {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_metadata(metadata: &BTreeMap<String, String>) -> Result<(), EvalManifestError> {
    for key in metadata.keys() {
        if is_secret_like_key(key) {
            return Err(EvalManifestError::SecretLikeMetadataKey { key: key.clone() });
        }
    }
    Ok(())
}

fn validate_permissions(permissions: &EvalPermissionProfile) -> Result<(), EvalManifestError> {
    for env_key in &permissions.env {
        validate_non_empty("permissions.env", env_key)?;
        if env_key.contains('=') {
            return Err(EvalManifestError::InvalidField {
                field: "permissions.env",
                message: "environment declarations must be keys only, not key=value pairs"
                    .to_string(),
            });
        }
    }
    Ok(())
}

fn validate_context_quality_expectations(
    kind: EvalScenarioKind,
    expectations: &EvalExpectations,
) -> Result<(), EvalManifestError> {
    let Some(context_quality) = expectations.context_quality.as_ref() else {
        if kind == EvalScenarioKind::ContextQuality {
            return Err(EvalManifestError::InvalidField {
                field: "expectations.context_quality",
                message: "context_quality expectations are required for context_quality scenarios"
                    .to_string(),
            });
        }
        return Ok(());
    };

    validate_non_empty("expectations.context_quality.task", &context_quality.task)?;
    validate_expectation_paths(
        "expectations.context_quality.required_selected_paths",
        &context_quality.required_selected_paths,
    )?;
    validate_expectation_paths(
        "expectations.context_quality.required_pinned_paths",
        &context_quality.required_pinned_paths,
    )?;
    Ok(())
}

fn validate_recovery_expectations(
    kind: EvalScenarioKind,
    expectations: &EvalExpectations,
) -> Result<(), EvalManifestError> {
    if kind == EvalScenarioKind::RecoveryCrashSmoke && expectations.recovery.is_none() {
        return Err(EvalManifestError::InvalidField {
            field: "expectations.recovery",
            message: "recovery expectations are required for recovery_crash_smoke scenarios"
                .to_string(),
        });
    }
    if let Some(recovery) = &expectations.recovery {
        if let Some(readiness) = &recovery.expected_readiness {
            validate_non_empty("expectations.recovery.expected_readiness", readiness)?;
        }
    }
    Ok(())
}

fn validate_team_workflow_expectations(
    kind: EvalScenarioKind,
    expectations: &EvalExpectations,
) -> Result<(), EvalManifestError> {
    if kind == EvalScenarioKind::TeamWorkflowSmoke && expectations.team_workflow.is_none() {
        return Err(EvalManifestError::InvalidField {
            field: "expectations.team_workflow",
            message: "team_workflow expectations are required for team_workflow_smoke scenarios"
                .to_string(),
        });
    }
    Ok(())
}

fn validate_memory_expectations(
    kind: EvalScenarioKind,
    expectations: &EvalExpectations,
) -> Result<(), EvalManifestError> {
    if kind == EvalScenarioKind::MemorySmoke && expectations.memory.is_none() {
        return Err(EvalManifestError::InvalidField {
            field: "expectations.memory",
            message: "memory expectations are required for memory_smoke scenarios".to_string(),
        });
    }
    Ok(())
}

fn validate_expectation_paths(
    field: &'static str,
    paths: &[String],
) -> Result<(), EvalManifestError> {
    for path in paths {
        validate_non_empty(field, path)?;
        let path = Utf8Path::new(path);
        reject_parent_components(field, path)?;
        if path.is_absolute() {
            return Err(EvalManifestError::UnsafePath {
                field,
                path: path.to_path_buf(),
                message: "absolute paths are not allowed".to_string(),
            });
        }
    }
    Ok(())
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
}

fn normalize_root<P>(fixture_root: P) -> Result<Utf8PathBuf, EvalManifestError>
where
    P: AsRef<Path>,
{
    let root = fixture_root.as_ref();
    if root.as_os_str().is_empty() {
        return Err(EvalManifestError::InvalidField {
            field: "fixture_root",
            message: "must not be empty".to_string(),
        });
    }

    let root = utf8_path_buf(root, "fixture_root")?;
    reject_parent_components("fixture_root", &root)?;
    clean_path(&root)
}

fn normalize_fixtures(
    manifest: &EvalScenarioManifest,
    root: &Utf8Path,
) -> Result<ValidatedEvalFixtures, EvalManifestError> {
    enforce_required_fixture(manifest.kind, &manifest.fixtures)?;

    Ok(ValidatedEvalFixtures {
        replay_trace: normalize_optional_fixture(
            "fixtures.replay_trace",
            root,
            manifest.fixtures.replay_trace.as_deref(),
        )?,
        evidence_bundle: normalize_optional_fixture(
            "fixtures.evidence_bundle",
            root,
            manifest.fixtures.evidence_bundle.as_deref(),
        )?,
        model_script: normalize_optional_fixture(
            "fixtures.model_script",
            root,
            manifest.fixtures.model_script.as_deref(),
        )?,
        workspace: normalize_optional_fixture(
            "fixtures.workspace",
            root,
            manifest.fixtures.workspace.as_deref(),
        )?,
        golden: normalize_optional_fixture(
            "expectations.golden",
            root,
            manifest.expectations.golden.as_deref(),
        )?,
    })
}

fn enforce_required_fixture(
    kind: EvalScenarioKind,
    fixtures: &EvalFixtures,
) -> Result<(), EvalManifestError> {
    let missing_field = match kind {
        EvalScenarioKind::ReplayTrace if fixtures.replay_trace.is_none() => {
            Some("fixtures.replay_trace")
        }
        EvalScenarioKind::EvidenceOnly if fixtures.evidence_bundle.is_none() => {
            Some("fixtures.evidence_bundle")
        }
        EvalScenarioKind::FakeModelTurn if fixtures.model_script.is_none() => {
            Some("fixtures.model_script")
        }
        EvalScenarioKind::LocalCoding if fixtures.workspace.is_none() => Some("fixtures.workspace"),
        EvalScenarioKind::ContextQuality if fixtures.workspace.is_none() => {
            Some("fixtures.workspace")
        }
        EvalScenarioKind::RecoveryCrashSmoke
        | EvalScenarioKind::TeamWorkflowSmoke
        | EvalScenarioKind::MemorySmoke => None,
        _ => None,
    };

    if let Some(field) = missing_field {
        return Err(EvalManifestError::MissingFixture { kind, field });
    }

    Ok(())
}

fn normalize_optional_fixture(
    field: &'static str,
    root: &Utf8Path,
    path: Option<&Utf8Path>,
) -> Result<Option<Utf8PathBuf>, EvalManifestError> {
    path.map(|path| normalize_fixture_path(field, root, path))
        .transpose()
}

fn normalize_fixture_path(
    field: &'static str,
    root: &Utf8Path,
    path: &Utf8Path,
) -> Result<Utf8PathBuf, EvalManifestError> {
    if path.as_str().trim().is_empty() {
        return Err(EvalManifestError::InvalidField {
            field,
            message: "path must not be empty".to_string(),
        });
    }

    reject_parent_components(field, path)?;

    let cleaned = clean_path(path)?;
    if cleaned.is_absolute() {
        if root.is_absolute() && cleaned.starts_with(root) {
            return Ok(cleaned);
        }
        return Err(EvalManifestError::UnsafePath {
            field,
            path: path.to_path_buf(),
            message: "absolute path is outside the fixture root".to_string(),
        });
    }

    Ok(root.join(cleaned))
}

fn reject_parent_components(field: &'static str, path: &Utf8Path) -> Result<(), EvalManifestError> {
    if path
        .as_std_path()
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EvalManifestError::UnsafePath {
            field,
            path: path.to_path_buf(),
            message: "path traversal with '..' is not allowed".to_string(),
        });
    }
    Ok(())
}

fn clean_path(path: &Utf8Path) -> Result<Utf8PathBuf, EvalManifestError> {
    let mut cleaned = PathBuf::new();
    for component in path.as_std_path().components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => unreachable!("parent components are rejected before cleaning"),
            Component::Normal(part) => cleaned.push(part),
            Component::RootDir => cleaned.push(component.as_os_str()),
            Component::Prefix(prefix) => cleaned.push(prefix.as_os_str()),
        }
    }
    Utf8PathBuf::from_path_buf(cleaned).map_err(|path| EvalManifestError::InvalidField {
        field: "path",
        message: format!("path is not valid UTF-8: {}", path.display()),
    })
}

fn utf8_path_buf(path: &Path, field: &'static str) -> Result<Utf8PathBuf, EvalManifestError> {
    Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|path| EvalManifestError::InvalidField {
        field,
        message: format!("path is not valid UTF-8: {}", path.display()),
    })
}
