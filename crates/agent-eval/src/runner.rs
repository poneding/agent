use std::collections::BTreeSet;
use std::fs;

use agent_core::{ObservabilityEvidenceBundle, verify_observability_replay_fixture};
use agent_storage::{check_replay_compatibility, load_replay_fixture_json};
use serde::{Deserialize, Serialize};

use crate::manifest::{
    EvalDiagnostic, EvalDiagnosticSeverity, EvalScenarioKind, ValidatedEvalScenario,
};
use crate::report::{
    EVAL_RUN_REPORT_SCHEMA_VERSION, EvalEvidenceSource, EvalEvidenceSummary, EvalRunReport,
    EvalRunSummary, EvalScenarioReport, EvalScenarioStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvalRunnerOptions {
    pub strictness: EvalStrictness,
    pub max_scenarios: Option<usize>,
    pub run_id: String,
}

impl Default for EvalRunnerOptions {
    fn default() -> Self {
        Self {
            strictness: EvalStrictness::Strict,
            max_scenarios: None,
            run_id: "eval-run".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalStrictness {
    Strict,
    Warn,
}

#[must_use]
pub fn run_eval_scenario(
    scenario: &ValidatedEvalScenario,
    options: EvalRunnerOptions,
) -> EvalScenarioReport {
    match scenario.manifest.kind {
        EvalScenarioKind::ReplayTrace => run_replay_trace_scenario(scenario),
        EvalScenarioKind::EvidenceOnly => run_evidence_only_scenario(scenario),
        EvalScenarioKind::FakeModelTurn | EvalScenarioKind::LocalCoding => {
            run_unimplemented_scenario(scenario, options.strictness)
        }
    }
}

#[must_use]
pub fn run_eval_scenarios(
    scenarios: &[ValidatedEvalScenario],
    options: EvalRunnerOptions,
) -> EvalRunReport {
    let mut reports = Vec::with_capacity(scenarios.len());
    let max_scenarios = options.max_scenarios.unwrap_or(usize::MAX);

    for (index, scenario) in scenarios.iter().enumerate() {
        if index >= max_scenarios {
            reports.push(not_run_report(scenario, "max_scenarios limit reached"));
            continue;
        }
        reports.push(run_eval_scenario(scenario, options.clone()));
    }

    let summary = EvalRunSummary::from_reports(&reports);
    EvalRunReport {
        schema_version: EVAL_RUN_REPORT_SCHEMA_VERSION,
        run_id: options.run_id,
        scenarios: reports,
        summary,
    }
}

fn run_replay_trace_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(path) = scenario.normalized_fixtures.replay_trace.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "fixture_missing",
                "validated scenario has no replay trace fixture",
                Some("fixtures.replay_trace"),
            ),
            None,
        );
    };

    let input = match fs::read_to_string(path.as_std_path()) {
        Ok(input) => input,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_read_failed",
                    format!("failed to read replay fixture: {error}"),
                    Some("fixtures.replay_trace"),
                ),
                None,
            );
        }
    };

    if let Err(error) = check_replay_compatibility(&input) {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "replay_compatibility_failed",
                format!("replay compatibility failed: {error}"),
                Some("fixtures.replay_trace"),
            ),
            None,
        );
    }

    let fixture = match load_replay_fixture_json(&input) {
        Ok(fixture) => fixture,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "replay_fixture_parse_failed",
                    format!("replay fixture parse failed: {error}"),
                    Some("fixtures.replay_trace"),
                ),
                None,
            );
        }
    };

    let verification = match verify_observability_replay_fixture(&fixture) {
        Ok(verification) => verification,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "replay_evidence_failed",
                    format!("replay evidence verification failed: {error}"),
                    Some("fixtures.replay_trace"),
                ),
                Some(EvalEvidenceSummary {
                    source: EvalEvidenceSource::ReplayTrace,
                    event_count: fixture.events.len(),
                    checkpoint_count: fixture.checkpoints.len(),
                    trace_entries: 0,
                    audit_records: 0,
                    telemetry_records: 0,
                    replayable: false,
                    diagnostics: 1,
                }),
            );
        }
    };

    let mut diagnostics = Vec::new();
    diagnostics.extend(required_event_diagnostics(scenario, &fixture.events));
    diagnostics.extend(verification.diagnostics.iter().map(|message| {
        diagnostic(
            EvalDiagnosticSeverity::Error,
            "replay_evidence_diagnostic",
            message,
            Some("fixtures.replay_trace"),
        )
    }));

    let summary = EvalEvidenceSummary {
        source: EvalEvidenceSource::ReplayTrace,
        event_count: verification.source_event_count,
        checkpoint_count: verification.source_checkpoint_count,
        trace_entries: verification.regenerated_trace_count,
        audit_records: verification.regenerated_audit_count,
        telemetry_records: 0,
        replayable: verification.replayable,
        diagnostics: diagnostics.len(),
    };
    let status = if verification.replayable && diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };

    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        diagnostics,
        evidence_summary: Some(summary),
    }
}

fn run_evidence_only_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(path) = scenario.normalized_fixtures.evidence_bundle.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "fixture_missing",
                "validated scenario has no evidence bundle fixture",
                Some("fixtures.evidence_bundle"),
            ),
            None,
        );
    };

    let input = match fs::read_to_string(path.as_std_path()) {
        Ok(input) => input,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_read_failed",
                    format!("failed to read evidence bundle: {error}"),
                    Some("fixtures.evidence_bundle"),
                ),
                None,
            );
        }
    };

    let bundle: ObservabilityEvidenceBundle = match serde_json::from_str(&input) {
        Ok(bundle) => bundle,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "evidence_bundle_parse_failed",
                    format!("evidence bundle parse failed: {error}"),
                    Some("fixtures.evidence_bundle"),
                ),
                None,
            );
        }
    };

    let mut diagnostics = Vec::new();
    if !bundle.summary.replayable {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "evidence_not_replayable",
            "evidence bundle summary is not replayable",
            Some("fixtures.evidence_bundle"),
        ));
    }

    let summary = EvalEvidenceSummary {
        source: EvalEvidenceSource::EvidenceBundle,
        event_count: bundle.summary.total_events,
        checkpoint_count: bundle.replay.checkpoint_count,
        trace_entries: bundle.summary.trace_entries,
        audit_records: bundle.summary.audit_records,
        telemetry_records: bundle.summary.telemetry_records,
        replayable: bundle.summary.replayable,
        diagnostics: diagnostics.len() + bundle.summary.diagnostics,
    };
    let status = if diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };

    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        diagnostics,
        evidence_summary: Some(summary),
    }
}

fn run_unimplemented_scenario(
    scenario: &ValidatedEvalScenario,
    strictness: EvalStrictness,
) -> EvalScenarioReport {
    let (status, severity) = match strictness {
        EvalStrictness::Strict => (EvalScenarioStatus::Invalid, EvalDiagnosticSeverity::Error),
        EvalStrictness::Warn => (EvalScenarioStatus::Skipped, EvalDiagnosticSeverity::Warning),
    };

    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        diagnostics: vec![diagnostic(
            severity,
            "scenario_kind_not_implemented",
            format!(
                "{:?} scenarios are reserved for a later runner implementation",
                scenario.manifest.kind
            ),
            Some("kind"),
        )],
        evidence_summary: None,
    }
}

fn not_run_report(
    scenario: &ValidatedEvalScenario,
    reason: impl Into<String>,
) -> EvalScenarioReport {
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status: EvalScenarioStatus::NotRun,
        diagnostics: vec![diagnostic(
            EvalDiagnosticSeverity::Warning,
            "scenario_not_run",
            reason,
            None,
        )],
        evidence_summary: None,
    }
}

fn failed_report(
    scenario: &ValidatedEvalScenario,
    diagnostic: EvalDiagnostic,
    evidence_summary: Option<EvalEvidenceSummary>,
) -> EvalScenarioReport {
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status: EvalScenarioStatus::Failed,
        diagnostics: vec![diagnostic],
        evidence_summary,
    }
}

fn required_event_diagnostics(
    scenario: &ValidatedEvalScenario,
    events: &[agent_types::RuntimeEvent],
) -> Vec<EvalDiagnostic> {
    if scenario.manifest.expectations.required_events.is_empty() {
        return Vec::new();
    }

    let present = events
        .iter()
        .filter_map(runtime_event_type)
        .collect::<BTreeSet<_>>();

    scenario
        .manifest
        .expectations
        .required_events
        .iter()
        .filter(|required| !present.contains(required.as_str()))
        .map(|required| {
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "required_event_missing",
                format!("required event '{required}' was not present in replay fixture"),
                Some("expectations.required_events"),
            )
        })
        .collect()
}

fn runtime_event_type(event: &agent_types::RuntimeEvent) -> Option<String> {
    serde_json::to_value(&event.payload).ok().and_then(|value| {
        value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    })
}

fn diagnostic(
    severity: EvalDiagnosticSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
    field: Option<&str>,
) -> EvalDiagnostic {
    EvalDiagnostic {
        severity,
        code: code.into(),
        message: bounded(message.into()),
        field: field.map(str::to_string),
    }
}

fn bounded(message: String) -> String {
    const MAX_DIAGNOSTIC_CHARS: usize = 512;
    let mut chars = message.chars();
    let bounded = chars
        .by_ref()
        .take(MAX_DIAGNOSTIC_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        format!("{bounded}...")
    } else {
        bounded
    }
}
