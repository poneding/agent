use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::manifest::{EvalManifestError, ValidatedEvalScenario, parse_eval_scenario_json};
use crate::runner::EvalStrictness;

pub const EVAL_SUITE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvalSuiteManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub default_strictness: EvalStrictness,
    pub scenarios: Vec<EvalSuiteScenarioRef>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvalSuiteScenarioRef {
    pub path: Utf8PathBuf,
    pub alias: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub strictness: Option<EvalStrictness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedEvalSuite {
    pub manifest: EvalSuiteManifest,
    pub normalized_suite_root: Utf8PathBuf,
    pub scenario_refs: Vec<ValidatedEvalSuiteScenarioRef>,
    pub scenarios: Vec<ValidatedEvalScenario>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedEvalSuiteScenarioRef {
    pub path: Utf8PathBuf,
    pub alias: Option<String>,
    pub labels: Vec<String>,
    pub strictness: EvalStrictness,
    pub normalized_path: Utf8PathBuf,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EvalSuiteError {
    #[error("failed to parse eval suite manifest: {0}")]
    Deserialize(String),
    #[error("unsupported eval suite schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { supported: u32, found: u32 },
    #[error("invalid eval suite field {field}: {message}")]
    InvalidField {
        field: &'static str,
        message: String,
    },
    #[error("duplicate eval suite scenario alias: {alias}")]
    DuplicateScenarioAlias { alias: String },
    #[error("unsafe path in {field}: {message}")]
    UnsafePath {
        field: &'static str,
        path: Utf8PathBuf,
        message: String,
    },
    #[error("missing eval scenario file {path}: {message}")]
    MissingScenarioFile { path: Utf8PathBuf, message: String },
    #[error("failed to read eval scenario file {path}: {message}")]
    ScenarioReadFailed { path: Utf8PathBuf, message: String },
    #[error("invalid eval scenario file {path}: {source}")]
    ScenarioManifest {
        path: Utf8PathBuf,
        source: EvalManifestError,
    },
}

pub fn parse_eval_suite_json<P>(
    input: &str,
    suite_root: P,
) -> Result<ValidatedEvalSuite, EvalSuiteError>
where
    P: AsRef<Path>,
{
    let manifest = serde_json::from_str(input)
        .map_err(|error| EvalSuiteError::Deserialize(error.to_string()))?;
    validate_eval_suite(manifest, suite_root)
}

pub fn validate_eval_suite<P>(
    manifest: EvalSuiteManifest,
    suite_root: P,
) -> Result<ValidatedEvalSuite, EvalSuiteError>
where
    P: AsRef<Path>,
{
    if manifest.schema_version != EVAL_SUITE_SCHEMA_VERSION {
        return Err(EvalSuiteError::UnsupportedSchemaVersion {
            supported: EVAL_SUITE_SCHEMA_VERSION,
            found: manifest.schema_version,
        });
    }

    validate_suite_id(&manifest.id)?;
    validate_non_empty("name", &manifest.name)?;
    validate_scenario_refs(&manifest.scenarios)?;

    let normalized_suite_root = normalize_root(suite_root)?;
    let scenario_refs = normalize_scenario_refs(&manifest, &normalized_suite_root)?;
    let scenarios = load_scenarios(&scenario_refs)?;

    Ok(ValidatedEvalSuite {
        manifest,
        normalized_suite_root,
        scenario_refs,
        scenarios,
    })
}

fn validate_scenario_refs(refs: &[EvalSuiteScenarioRef]) -> Result<(), EvalSuiteError> {
    if refs.is_empty() {
        return Err(EvalSuiteError::InvalidField {
            field: "scenarios",
            message: "must contain at least one scenario".to_string(),
        });
    }

    let mut aliases = BTreeSet::new();
    for scenario_ref in refs {
        if let Some(alias) = scenario_ref.alias.as_deref() {
            validate_alias(alias)?;
            if !aliases.insert(alias.to_string()) {
                return Err(EvalSuiteError::DuplicateScenarioAlias {
                    alias: alias.to_string(),
                });
            }
        }

        validate_labels(&scenario_ref.labels)?;
    }

    Ok(())
}

fn normalize_scenario_refs(
    manifest: &EvalSuiteManifest,
    root: &Utf8Path,
) -> Result<Vec<ValidatedEvalSuiteScenarioRef>, EvalSuiteError> {
    manifest
        .scenarios
        .iter()
        .map(|scenario_ref| {
            let normalized_path = normalize_suite_path("scenarios.path", root, &scenario_ref.path)?;
            Ok(ValidatedEvalSuiteScenarioRef {
                path: scenario_ref.path.clone(),
                alias: scenario_ref.alias.clone(),
                labels: scenario_ref.labels.clone(),
                strictness: scenario_ref
                    .strictness
                    .unwrap_or(manifest.default_strictness),
                normalized_path,
            })
        })
        .collect()
}

fn load_scenarios(
    refs: &[ValidatedEvalSuiteScenarioRef],
) -> Result<Vec<ValidatedEvalScenario>, EvalSuiteError> {
    refs.iter()
        .map(|scenario_ref| {
            let input = fs::read_to_string(scenario_ref.normalized_path.as_std_path()).map_err(
                |error| match error.kind() {
                    ErrorKind::NotFound => EvalSuiteError::MissingScenarioFile {
                        path: scenario_ref.normalized_path.clone(),
                        message: "scenario manifest is required by suite".to_string(),
                    },
                    _ => EvalSuiteError::ScenarioReadFailed {
                        path: scenario_ref.normalized_path.clone(),
                        message: error.to_string(),
                    },
                },
            )?;

            let fixture_root = scenario_ref
                .normalized_path
                .parent()
                .unwrap_or_else(|| Utf8Path::new(""));
            parse_eval_scenario_json(&input, fixture_root).map_err(|source| {
                EvalSuiteError::ScenarioManifest {
                    path: scenario_ref.normalized_path.clone(),
                    source,
                }
            })
        })
        .collect()
}

fn validate_suite_id(id: &str) -> Result<(), EvalSuiteError> {
    validate_non_empty("id", id)?;

    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return Err(EvalSuiteError::InvalidField {
            field: "id",
            message: "must not be empty".to_string(),
        });
    };

    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err(EvalSuiteError::InvalidField {
            field: "id",
            message: "must start with a lowercase letter or digit".to_string(),
        });
    }

    if !id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(EvalSuiteError::InvalidField {
            field: "id",
            message: "must contain only lowercase letters, digits, '-' or '_'".to_string(),
        });
    }

    Ok(())
}

fn validate_alias(alias: &str) -> Result<(), EvalSuiteError> {
    validate_non_empty("scenarios.alias", alias)?;

    if !alias
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(EvalSuiteError::InvalidField {
            field: "scenarios.alias",
            message: "must contain only lowercase letters, digits, '-' or '_'".to_string(),
        });
    }

    Ok(())
}

fn validate_labels(labels: &[String]) -> Result<(), EvalSuiteError> {
    let mut seen = BTreeSet::new();
    for label in labels {
        validate_non_empty("scenarios.labels", label)?;
        if is_secret_like(label) {
            return Err(EvalSuiteError::InvalidField {
                field: "scenarios.labels",
                message: "label looks secret-like and cannot be stored".to_string(),
            });
        }
        if !label
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(EvalSuiteError::InvalidField {
                field: "scenarios.labels",
                message: "labels must contain only lowercase letters, digits, '-' or '_'"
                    .to_string(),
            });
        }
        if !seen.insert(label) {
            return Err(EvalSuiteError::InvalidField {
                field: "scenarios.labels",
                message: "labels must not repeat within one scenario ref".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), EvalSuiteError> {
    if value.trim().is_empty() {
        return Err(EvalSuiteError::InvalidField {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn is_secret_like(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '.'], "_");
    normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("password")
        || normalized.contains("credential")
        || normalized.contains("private_key")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("sk_")
}

fn normalize_root<P>(suite_root: P) -> Result<Utf8PathBuf, EvalSuiteError>
where
    P: AsRef<Path>,
{
    let root = suite_root.as_ref();
    if root.as_os_str().is_empty() {
        return Err(EvalSuiteError::InvalidField {
            field: "suite_root",
            message: "must not be empty".to_string(),
        });
    }

    let root = utf8_path_buf(root, "suite_root")?;
    reject_parent_components("suite_root", &root)?;
    clean_path(&root)
}

fn normalize_suite_path(
    field: &'static str,
    root: &Utf8Path,
    path: &Utf8Path,
) -> Result<Utf8PathBuf, EvalSuiteError> {
    if path.as_str().trim().is_empty() {
        return Err(EvalSuiteError::InvalidField {
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
        return Err(EvalSuiteError::UnsafePath {
            field,
            path: path.to_path_buf(),
            message: "absolute path is outside the suite root".to_string(),
        });
    }

    Ok(root.join(cleaned))
}

fn reject_parent_components(field: &'static str, path: &Utf8Path) -> Result<(), EvalSuiteError> {
    if path
        .as_std_path()
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EvalSuiteError::UnsafePath {
            field,
            path: path.to_path_buf(),
            message: "path traversal with '..' is not allowed".to_string(),
        });
    }
    Ok(())
}

fn clean_path(path: &Utf8Path) -> Result<Utf8PathBuf, EvalSuiteError> {
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
    Utf8PathBuf::from_path_buf(cleaned).map_err(|path| EvalSuiteError::InvalidField {
        field: "path",
        message: format!("path is not valid UTF-8: {}", path.display()),
    })
}

fn utf8_path_buf(path: &Path, field: &'static str) -> Result<Utf8PathBuf, EvalSuiteError> {
    Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|path| EvalSuiteError::InvalidField {
        field,
        message: format!("path is not valid UTF-8: {}", path.display()),
    })
}
