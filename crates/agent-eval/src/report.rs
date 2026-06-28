use serde::{Deserialize, Serialize};

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
}
