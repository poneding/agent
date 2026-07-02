use std::fs;
use std::path::Component;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::manifest::EvalDiagnostic;

pub const EVAL_RUN_REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalRunReport {
    pub schema_version: u32,
    pub run_id: String,
    pub scenarios: Vec<EvalScenarioReport>,
    pub summary: EvalRunSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalArtifactOptions {
    pub output_dir: Utf8PathBuf,
    pub write_json: bool,
    pub write_summary: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalArtifactSet {
    pub json_report: Option<Utf8PathBuf>,
    pub summary_report: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum EvalReportArtifactError {
    #[error("eval artifact output_dir must not be empty")]
    EmptyOutputDir,
    #[error("eval artifact output_dir is unsafe: {path}")]
    UnsafeOutputDir { path: Utf8PathBuf },
    #[error("eval artifact writer must select at least one artifact")]
    NoArtifactsSelected,
    #[error("failed to serialize eval run report: {message}")]
    Json { message: String },
    #[error("failed to write eval artifact {path}: {message}")]
    Io { path: Utf8PathBuf, message: String },
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalRunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub invalid: usize,
    pub not_run: usize,
}

impl EvalRunSummary {
    #[must_use]
    pub fn from_reports(reports: &[EvalScenarioReport]) -> Self {
        let mut summary = Self {
            total: reports.len(),
            ..Self::default()
        };
        for report in reports {
            match report.status {
                EvalScenarioStatus::Passed => summary.passed += 1,
                EvalScenarioStatus::Failed => summary.failed += 1,
                EvalScenarioStatus::Skipped => summary.skipped += 1,
                EvalScenarioStatus::Invalid => summary.invalid += 1,
                EvalScenarioStatus::NotRun => summary.not_run += 1,
            }
        }
        summary
    }
}

pub fn write_eval_artifacts(
    report: &EvalRunReport,
    options: EvalArtifactOptions,
) -> Result<EvalArtifactSet, EvalReportArtifactError> {
    validate_artifact_options(&options)?;
    fs::create_dir_all(options.output_dir.as_std_path())
        .map_err(|error| artifact_io_error(options.output_dir.clone(), error))?;

    let mut artifacts = EvalArtifactSet::default();
    if options.write_json {
        let path = options.output_dir.join("eval-run-report.json");
        let json = serde_json::to_string_pretty(report).map_err(|error| {
            EvalReportArtifactError::Json {
                message: error.to_string(),
            }
        })?;
        write_atomic(&path, &(json + "\n"))?;
        artifacts.json_report = Some(path);
    }

    if options.write_summary {
        let path = options.output_dir.join("eval-summary.txt");
        write_atomic(&path, &render_eval_summary_text(report))?;
        artifacts.summary_report = Some(path);
    }

    Ok(artifacts)
}

#[must_use]
pub fn render_eval_summary_text(report: &EvalRunReport) -> String {
    let mut output = format!(
        "Eval run {}\ntotal={} passed={} failed={} skipped={} invalid={} not_run={}\n",
        report.run_id,
        report.summary.total,
        report.summary.passed,
        report.summary.failed,
        report.summary.skipped,
        report.summary.invalid,
        report.summary.not_run
    );
    for scenario in &report.scenarios {
        output.push_str(&format!(
            "{} status={}\n",
            scenario.scenario_id,
            scenario_status_label(scenario.status)
        ));
    }
    output
}

fn validate_artifact_options(options: &EvalArtifactOptions) -> Result<(), EvalReportArtifactError> {
    if options.output_dir.as_str().trim().is_empty() {
        return Err(EvalReportArtifactError::EmptyOutputDir);
    }
    if !options.write_json && !options.write_summary {
        return Err(EvalReportArtifactError::NoArtifactsSelected);
    }
    if options
        .output_dir
        .as_std_path()
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EvalReportArtifactError::UnsafeOutputDir {
            path: options.output_dir.clone(),
        });
    }
    Ok(())
}

fn write_atomic(path: &Utf8Path, content: &str) -> Result<(), EvalReportArtifactError> {
    let parent = path.parent().unwrap_or_else(|| Utf8Path::new("."));
    fs::create_dir_all(parent.as_std_path())
        .map_err(|error| artifact_io_error(parent.to_path_buf(), error))?;
    let file_name = path.file_name().unwrap_or("eval-artifact");
    let temp_path = path.with_file_name(format!(".{file_name}.tmp"));
    fs::write(temp_path.as_std_path(), content)
        .map_err(|error| artifact_io_error(temp_path.clone(), error))?;

    match fs::rename(temp_path.as_std_path(), path.as_std_path()) {
        Ok(()) => Ok(()),
        Err(_error) if path.as_std_path().exists() => {
            fs::remove_file(path.as_std_path())
                .map_err(|remove_error| artifact_io_error(path.to_path_buf(), remove_error))?;
            fs::rename(temp_path.as_std_path(), path.as_std_path())
                .map_err(|rename_error| artifact_io_error(path.to_path_buf(), rename_error))
        }
        Err(error) => Err(artifact_io_error(path.to_path_buf(), error)),
    }
}

fn artifact_io_error(path: Utf8PathBuf, error: std::io::Error) -> EvalReportArtifactError {
    EvalReportArtifactError::Io {
        path,
        message: error.to_string(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalScenarioReport {
    pub scenario_id: String,
    pub status: EvalScenarioStatus,
    pub diagnostics: Vec<EvalDiagnostic>,
    pub evidence_summary: Option<EvalEvidenceSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalScenarioStatus {
    Passed,
    Failed,
    Skipped,
    Invalid,
    NotRun,
}

fn scenario_status_label(status: EvalScenarioStatus) -> &'static str {
    match status {
        EvalScenarioStatus::Passed => "passed",
        EvalScenarioStatus::Failed => "failed",
        EvalScenarioStatus::Skipped => "skipped",
        EvalScenarioStatus::Invalid => "invalid",
        EvalScenarioStatus::NotRun => "not_run",
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalEvidenceSummary {
    pub source: EvalEvidenceSource,
    pub event_count: usize,
    pub checkpoint_count: usize,
    pub trace_entries: usize,
    pub audit_records: usize,
    pub telemetry_records: usize,
    pub replayable: bool,
    pub diagnostics: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalEvidenceSource {
    ReplayTrace,
    EvidenceBundle,
    LocalCoding,
    ContextQuality,
    RecoveryCrashSmoke,
    TeamWorkflowSmoke,
}
