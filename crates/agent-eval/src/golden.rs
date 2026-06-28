use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::manifest::{EvalDiagnostic, EvalDiagnosticSeverity};
use crate::report::{EvalRunReport, EvalScenarioReport, EvalScenarioStatus};
use crate::suite::ValidatedEvalSuite;

const DEFAULT_MAX_VALUE_CHARS: usize = 256;
const REDACTED: &str = "[redacted]";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldenComparisonOptions {
    pub update_mode: bool,
    pub max_value_chars: usize,
}

impl Default for GoldenComparisonOptions {
    fn default() -> Self {
        Self {
            update_mode: false,
            max_value_chars: DEFAULT_MAX_VALUE_CHARS,
        }
    }
}

impl GoldenComparisonOptions {
    fn validate(self) -> Result<Self, GoldenComparisonError> {
        if self.max_value_chars == 0 {
            return Err(GoldenComparisonError::InvalidMaxValueChars);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenComparisonResult {
    pub status: EvalScenarioStatus,
    pub mismatches: Vec<GoldenMismatch>,
    pub normalized_expected: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenMismatch {
    pub path: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GoldenComparisonError {
    #[error("failed to serialize eval report for golden comparison: {message}")]
    Serialize { message: String },
    #[error("golden JSON is malformed: {message}")]
    MalformedGolden { message: String },
    #[error("failed to read golden file: {message}")]
    Read { message: String },
    #[error("golden comparison max_value_chars must be greater than zero")]
    InvalidMaxValueChars,
}

pub fn normalized_report_golden(
    report: &EvalScenarioReport,
) -> Result<Value, GoldenComparisonError> {
    let value = serde_json::to_value(report).map_err(|error| GoldenComparisonError::Serialize {
        message: error.to_string(),
    })?;
    Ok(redact_json("", &value))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldenUpdateOptions {
    pub dry_run: bool,
}

impl Default for GoldenUpdateOptions {
    fn default() -> Self {
        Self { dry_run: true }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenUpdateReport {
    pub updated: usize,
    pub unchanged: usize,
    pub diagnostics: Vec<EvalDiagnostic>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GoldenUpdateError {
    #[error("missing eval run report for scenario {scenario_id}")]
    MissingScenarioReport { scenario_id: String },
    #[error("scenario {scenario_id} has no configured golden path")]
    MissingGoldenPath { scenario_id: String },
    #[error("unsafe golden path {path}: {message}")]
    UnsafeGoldenPath { path: Utf8PathBuf, message: String },
    #[error("existing golden JSON is malformed at {path}: {message}")]
    MalformedExistingGolden { path: Utf8PathBuf, message: String },
    #[error("failed to serialize updated golden: {message}")]
    Serialize { message: String },
    #[error("failed to read golden {path}: {message}")]
    Read { path: Utf8PathBuf, message: String },
    #[error("failed to write golden {path}: {message}")]
    Write { path: Utf8PathBuf, message: String },
    #[error("failed to normalize report for golden update: {source}")]
    Normalize { source: GoldenComparisonError },
}

pub fn update_suite_goldens(
    suite: &ValidatedEvalSuite,
    run_report: &EvalRunReport,
    options: GoldenUpdateOptions,
) -> Result<GoldenUpdateReport, GoldenUpdateError> {
    let reports = run_report
        .scenarios
        .iter()
        .map(|report| (report.scenario_id.as_str(), report))
        .collect::<BTreeMap<_, _>>();

    let mut update_report = GoldenUpdateReport {
        updated: 0,
        unchanged: 0,
        diagnostics: Vec::new(),
    };

    for scenario in &suite.scenarios {
        let scenario_id = scenario.manifest.id.as_str();
        let scenario_report =
            reports
                .get(scenario_id)
                .ok_or_else(|| GoldenUpdateError::MissingScenarioReport {
                    scenario_id: scenario_id.to_string(),
                })?;
        let golden_path = scenario
            .normalized_fixtures
            .golden
            .as_ref()
            .ok_or_else(|| GoldenUpdateError::MissingGoldenPath {
                scenario_id: scenario_id.to_string(),
            })?;
        validate_golden_path(&suite.normalized_suite_root, golden_path)?;

        let normalized = normalized_report_golden(scenario_report)
            .map_err(|source| GoldenUpdateError::Normalize { source })?;
        let existing = read_existing_golden(golden_path)?;
        if existing.as_ref() == Some(&normalized) {
            update_report.unchanged += 1;
            continue;
        }

        update_report.updated += 1;
        let code = if options.dry_run {
            "golden_update_dry_run"
        } else {
            "golden_updated"
        };
        let message = if options.dry_run {
            format!("would update golden for scenario {scenario_id}")
        } else {
            format!("updated golden for scenario {scenario_id}")
        };
        update_report.diagnostics.push(update_diagnostic(
            EvalDiagnosticSeverity::Info,
            code,
            message,
            golden_path,
        ));

        if !options.dry_run {
            let json = serde_json::to_string_pretty(&normalized).map_err(|error| {
                GoldenUpdateError::Serialize {
                    message: error.to_string(),
                }
            })?;
            write_atomic(golden_path, &(json + "\n"))?;
        }
    }

    Ok(update_report)
}

pub fn compare_report_to_golden_json(
    report: &EvalScenarioReport,
    golden_json: &str,
    options: GoldenComparisonOptions,
) -> Result<GoldenComparisonResult, GoldenComparisonError> {
    let options = options.validate()?;
    let actual = normalized_report_golden(report)?;

    if options.update_mode {
        return Ok(update_result(actual));
    }

    let expected = serde_json::from_str(golden_json).map_err(|error| {
        GoldenComparisonError::MalformedGolden {
            message: error.to_string(),
        }
    })?;
    Ok(compare_values(&expected, &actual, options))
}

pub fn compare_report_to_golden_file<P>(
    report: &EvalScenarioReport,
    path: P,
    options: GoldenComparisonOptions,
) -> Result<GoldenComparisonResult, GoldenComparisonError>
where
    P: AsRef<Path>,
{
    let options = options.validate()?;
    let actual = normalized_report_golden(report)?;

    if options.update_mode {
        return Ok(update_result(actual));
    }

    let golden_json = match fs::read_to_string(path.as_ref()) {
        Ok(input) => input,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(GoldenComparisonResult {
                status: EvalScenarioStatus::Failed,
                mismatches: vec![GoldenMismatch {
                    path: String::new(),
                    expected: "<golden file>".to_string(),
                    actual: "<missing>".to_string(),
                }],
                normalized_expected: None,
            });
        }
        Err(error) => {
            return Err(GoldenComparisonError::Read {
                message: error.to_string(),
            });
        }
    };

    let expected = serde_json::from_str(&golden_json).map_err(|error| {
        GoldenComparisonError::MalformedGolden {
            message: error.to_string(),
        }
    })?;
    Ok(compare_values(&expected, &actual, options))
}

fn update_result(actual: Value) -> GoldenComparisonResult {
    GoldenComparisonResult {
        status: EvalScenarioStatus::Passed,
        mismatches: Vec::new(),
        normalized_expected: Some(actual),
    }
}

fn validate_golden_path(root: &Utf8Path, path: &Utf8Path) -> Result<(), GoldenUpdateError> {
    if path.as_str().trim().is_empty() {
        return Err(GoldenUpdateError::UnsafeGoldenPath {
            path: path.to_path_buf(),
            message: "path must not be empty".to_string(),
        });
    }
    if path
        .as_std_path()
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(GoldenUpdateError::UnsafeGoldenPath {
            path: path.to_path_buf(),
            message: "path traversal with '..' is not allowed".to_string(),
        });
    }
    if !path.starts_with(root) {
        return Err(GoldenUpdateError::UnsafeGoldenPath {
            path: path.to_path_buf(),
            message: "path is outside the suite root".to_string(),
        });
    }
    Ok(())
}

fn read_existing_golden(path: &Utf8Path) -> Result<Option<Value>, GoldenUpdateError> {
    let input = match fs::read_to_string(path.as_std_path()) {
        Ok(input) => input,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(GoldenUpdateError::Read {
                path: path.to_path_buf(),
                message: error.to_string(),
            });
        }
    };

    let value = serde_json::from_str(&input).map_err(|error| {
        GoldenUpdateError::MalformedExistingGolden {
            path: path.to_path_buf(),
            message: error.to_string(),
        }
    })?;
    Ok(Some(value))
}

fn write_atomic(path: &Utf8Path, content: &str) -> Result<(), GoldenUpdateError> {
    let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
    fs::create_dir_all(parent.as_std_path()).map_err(|error| GoldenUpdateError::Write {
        path: parent.to_path_buf(),
        message: error.to_string(),
    })?;

    let file_name = path.file_name().unwrap_or("golden.json");
    let temp_path = path.with_file_name(format!(".{file_name}.tmp"));
    fs::write(temp_path.as_std_path(), content).map_err(|error| GoldenUpdateError::Write {
        path: temp_path.clone(),
        message: error.to_string(),
    })?;

    match fs::rename(temp_path.as_std_path(), path.as_std_path()) {
        Ok(()) => Ok(()),
        Err(_error) if path.as_std_path().exists() => {
            fs::remove_file(path.as_std_path()).map_err(|remove_error| {
                GoldenUpdateError::Write {
                    path: path.to_path_buf(),
                    message: remove_error.to_string(),
                }
            })?;
            fs::rename(temp_path.as_std_path(), path.as_std_path()).map_err(|rename_error| {
                GoldenUpdateError::Write {
                    path: path.to_path_buf(),
                    message: rename_error.to_string(),
                }
            })
        }
        Err(error) => Err(GoldenUpdateError::Write {
            path: path.to_path_buf(),
            message: error.to_string(),
        }),
    }
}

fn update_diagnostic(
    severity: EvalDiagnosticSeverity,
    code: &str,
    message: String,
    path: &Utf8Path,
) -> EvalDiagnostic {
    EvalDiagnostic {
        severity,
        code: code.to_string(),
        message,
        field: Some(path.to_string()),
    }
}

fn compare_values(
    expected: &Value,
    actual: &Value,
    options: GoldenComparisonOptions,
) -> GoldenComparisonResult {
    let mut mismatches = Vec::new();
    compare_selected(
        String::new(),
        expected,
        Some(actual),
        &mut mismatches,
        options,
    );
    let status = if mismatches.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };
    GoldenComparisonResult {
        status,
        mismatches,
        normalized_expected: None,
    }
}

fn compare_selected(
    path: String,
    expected: &Value,
    actual: Option<&Value>,
    mismatches: &mut Vec<GoldenMismatch>,
    options: GoldenComparisonOptions,
) {
    let Some(actual) = actual else {
        mismatches.push(mismatch(&path, Some(expected), None, options));
        return;
    };

    match (expected, actual) {
        (Value::Object(expected_map), Value::Object(actual_map)) => {
            for (key, expected_value) in expected_map {
                compare_selected(
                    pointer_child(&path, key),
                    expected_value,
                    actual_map.get(key),
                    mismatches,
                    options,
                );
            }
        }
        (Value::Array(expected_items), Value::Array(actual_items)) => {
            for (index, expected_value) in expected_items.iter().enumerate() {
                compare_selected(
                    pointer_child(&path, &index.to_string()),
                    expected_value,
                    actual_items.get(index),
                    mismatches,
                    options,
                );
            }
        }
        _ if expected == actual => {}
        _ => mismatches.push(mismatch(&path, Some(expected), Some(actual), options)),
    }
}

fn mismatch(
    path: &str,
    expected: Option<&Value>,
    actual: Option<&Value>,
    options: GoldenComparisonOptions,
) -> GoldenMismatch {
    GoldenMismatch {
        path: path.to_string(),
        expected: render_value(path, expected, options.max_value_chars),
        actual: render_value(path, actual, options.max_value_chars),
    }
}

fn render_value(path: &str, value: Option<&Value>, max_chars: usize) -> String {
    let Some(value) = value else {
        return "<missing>".to_string();
    };
    let redacted = redact_json(path, value);
    let rendered =
        serde_json::to_string(&redacted).unwrap_or_else(|_| "<unrenderable>".to_string());
    bound(rendered, max_chars)
}

fn redact_json(path: &str, value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::new();
            for (key, value) in map {
                if is_secret_like(key) {
                    redacted.insert(key.clone(), Value::String(REDACTED.to_string()));
                } else {
                    redacted.insert(key.clone(), redact_json(&pointer_child(path, key), value));
                }
            }
            Value::Object(redacted)
        }
        Value::Array(items) => Value::Array(
            items
                .iter()
                .enumerate()
                .map(|(index, value)| redact_json(&pointer_child(path, &index.to_string()), value))
                .collect(),
        ),
        Value::String(text) if is_secret_like(path) || is_secret_like(text) => {
            Value::String(REDACTED.to_string())
        }
        _ => value.clone(),
    }
}

fn pointer_child(parent: &str, token: &str) -> String {
    let escaped = token.replace('~', "~0").replace('/', "~1");
    if parent.is_empty() {
        format!("/{escaped}")
    } else {
        format!("{parent}/{escaped}")
    }
}

fn bound(value: String, max_chars: usize) -> String {
    let mut chars = value.chars();
    let bounded = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}

fn is_secret_like(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("secret")
        || value.contains("token")
        || value.contains("password")
        || value.contains("credential")
        || value.contains("private_key")
        || value.contains("api_key")
        || value.contains("apikey")
        || value.contains("sk-")
}
