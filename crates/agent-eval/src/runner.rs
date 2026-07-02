use std::collections::BTreeSet;
use std::fs;

use agent_core::{
    ContextBudget, ContextRetrievalQuery, ObservabilityEvidenceBundle,
    ObservabilityEvidenceOptions, PinnedProjectContextPin, PinnedProjectContextRequest,
    ProjectContextLimits, ProjectContextPinnedRetrievalInput, ProjectContextRetrievalInput,
    build_observability_evidence, replay_final_review, retrieve_project_context_pinned_quality,
    retrieve_project_context_quality, run_local_coding_fixture, run_recovery_crash_smoke_fixture,
    verify_observability_replay_fixture,
};
use agent_storage::{check_replay_compatibility, load_replay_fixture_json};
use serde::{Deserialize, Serialize};

use crate::manifest::{
    EvalContextQualityExpectations, EvalDiagnostic, EvalDiagnosticSeverity, EvalScenarioKind,
    ValidatedEvalScenario,
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
        EvalScenarioKind::LocalCoding => run_local_coding_scenario(scenario),
        EvalScenarioKind::ContextQuality => run_context_quality_scenario(scenario),
        EvalScenarioKind::RecoveryCrashSmoke => run_recovery_crash_smoke_scenario(scenario),
        EvalScenarioKind::FakeModelTurn => run_unimplemented_scenario(scenario, options.strictness),
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

fn run_local_coding_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(path) = scenario.normalized_fixtures.workspace.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "fixture_missing",
                "validated scenario has no workspace fixture",
                Some("fixtures.workspace"),
            ),
            None,
        );
    };

    match fs::metadata(path.as_std_path()) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_invalid",
                    "workspace fixture must be a directory",
                    Some("fixtures.workspace"),
                ),
                None,
            );
        }
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_read_failed",
                    format!("failed to read workspace fixture: {error}"),
                    Some("fixtures.workspace"),
                ),
                None,
            );
        }
    }

    let run = match run_local_coding_fixture() {
        Ok(run) => run,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "local_coding_fixture_failed",
                    format!("local coding fixture failed: {error}"),
                    Some("kind"),
                ),
                None,
            );
        }
    };

    let review = match replay_final_review(
        &run.events,
        Some(format!("eval-local-coding-{}", scenario.manifest.id)),
    ) {
        Ok(review) => review,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "local_coding_replay_failed",
                    format!("local coding final review replay failed: {error}"),
                    Some("kind"),
                ),
                Some(EvalEvidenceSummary {
                    source: EvalEvidenceSource::LocalCoding,
                    event_count: run.events.len(),
                    checkpoint_count: 0,
                    trace_entries: 0,
                    audit_records: 0,
                    telemetry_records: 0,
                    replayable: false,
                    diagnostics: 1,
                }),
            );
        }
    };

    let evidence = match build_observability_evidence(
        &run.events,
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "local_coding_evidence_failed",
                    format!("local coding evidence build failed: {error}"),
                    Some("kind"),
                ),
                Some(EvalEvidenceSummary {
                    source: EvalEvidenceSource::LocalCoding,
                    event_count: run.events.len(),
                    checkpoint_count: 0,
                    trace_entries: review.trace_event_count,
                    audit_records: review.audit_event_count,
                    telemetry_records: 0,
                    replayable: review.replay_reconstructed,
                    diagnostics: 1,
                }),
            );
        }
    };

    let mut diagnostics = Vec::new();
    diagnostics.extend(required_event_diagnostics(scenario, &run.events));

    if review.changed_files.is_empty() {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "local_coding_changed_files_missing",
            "local coding final review had no changed-file evidence",
            Some("final_review.changed_files"),
        ));
    }
    if review.command_results.is_empty() {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "local_coding_command_evidence_missing",
            "local coding final review had no command/test evidence",
            Some("final_review.command_results"),
        ));
    }
    if review.trace_event_count == 0 || evidence.summary.trace_entries == 0 {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "local_coding_trace_missing",
            "local coding evidence had no trace entries",
            Some("evidence.trace"),
        ));
    }
    if review.audit_event_count == 0 {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "local_coding_audit_missing",
            "local coding final review had no audit evidence",
            Some("final_review.audit_event_count"),
        ));
    }
    if !review.replay_reconstructed || !evidence.summary.replayable {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "local_coding_replay_not_reconstructed",
            "local coding evidence was not replayable",
            Some("evidence.replay"),
        ));
    }
    diagnostics.extend(review.diagnostics.iter().map(|diagnostic| {
        crate::manifest::EvalDiagnostic {
            severity: EvalDiagnosticSeverity::Error,
            code: diagnostic.code.clone(),
            message: bounded(diagnostic.message.clone()),
            field: Some("final_review.diagnostics".to_string()),
        }
    }));

    let replayable = review.replay_reconstructed && evidence.summary.replayable;
    let summary = EvalEvidenceSummary {
        source: EvalEvidenceSource::LocalCoding,
        event_count: evidence.summary.total_events,
        checkpoint_count: evidence.replay.checkpoint_count,
        trace_entries: evidence.summary.trace_entries,
        audit_records: review.audit_event_count,
        telemetry_records: evidence.summary.telemetry_records,
        replayable,
        diagnostics: diagnostics.len(),
    };
    let status = if replayable && diagnostics.is_empty() {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContextQualityEvalOutcome {
    selected_paths: BTreeSet<String>,
    pinned_paths: BTreeSet<String>,
    retrieval_included: Option<bool>,
    signal_count: usize,
    trace_diagnostic_count: usize,
}

fn run_context_quality_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(path) = scenario.normalized_fixtures.workspace.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "fixture_missing",
                "validated scenario has no workspace fixture",
                Some("fixtures.workspace"),
            ),
            None,
        );
    };

    match fs::metadata(path.as_std_path()) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_invalid",
                    "workspace fixture must be a directory",
                    Some("fixtures.workspace"),
                ),
                None,
            );
        }
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "fixture_read_failed",
                    format!("failed to read workspace fixture: {error}"),
                    Some("fixtures.workspace"),
                ),
                None,
            );
        }
    }

    let Some(expectations) = scenario.manifest.expectations.context_quality.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "context_quality_expectations_missing",
                "context quality expectations are required",
                Some("expectations.context_quality"),
            ),
            None,
        );
    };

    let outcome = match evaluate_context_quality_workspace(path.as_std_path(), expectations) {
        Ok(outcome) => outcome,
        Err(message) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "context_quality_retrieval_failed",
                    message,
                    Some("expectations.context_quality"),
                ),
                None,
            );
        }
    };

    let mut diagnostics = Vec::new();
    for required_path in &expectations.required_selected_paths {
        if !outcome.selected_paths.contains(required_path) {
            diagnostics.push(diagnostic(
                EvalDiagnosticSeverity::Error,
                "context_quality_selected_path_missing",
                format!("required selected path '{required_path}' was not selected"),
                Some("expectations.context_quality.required_selected_paths"),
            ));
        }
    }
    for required_path in &expectations.required_pinned_paths {
        if !outcome.pinned_paths.contains(required_path) {
            diagnostics.push(diagnostic(
                EvalDiagnosticSeverity::Error,
                "context_quality_pinned_path_missing",
                format!("required pinned path '{required_path}' was not included"),
                Some("expectations.context_quality.required_pinned_paths"),
            ));
        }
    }
    if let Some(max_diagnostics) = expectations.max_diagnostics {
        if outcome.trace_diagnostic_count > max_diagnostics {
            diagnostics.push(diagnostic(
                EvalDiagnosticSeverity::Error,
                "context_quality_diagnostic_budget_exceeded",
                format!(
                    "context quality trace had {} diagnostics, expected at most {}",
                    outcome.trace_diagnostic_count, max_diagnostics
                ),
                Some("expectations.context_quality.max_diagnostics"),
            ));
        }
    }
    if expectations.require_retrieval_included && outcome.retrieval_included != Some(true) {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "context_quality_retrieval_layer_missing",
            "pinned context did not include the retrieval layer",
            Some("expectations.context_quality.require_retrieval_included"),
        ));
    }

    let diagnostic_count = diagnostics.len() + outcome.trace_diagnostic_count;
    let status = if diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        diagnostics,
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::ContextQuality,
            event_count: 0,
            checkpoint_count: 0,
            trace_entries: outcome.signal_count,
            audit_records: 0,
            telemetry_records: 0,
            replayable: true,
            diagnostics: diagnostic_count,
        }),
    }
}

fn run_recovery_crash_smoke_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(expectations) = scenario.manifest.expectations.recovery.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "recovery_expectations_missing",
                "recovery expectations are required",
                Some("expectations.recovery"),
            ),
            None,
        );
    };

    let smoke = match run_recovery_crash_smoke_fixture() {
        Ok(smoke) => smoke,
        Err(error) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "recovery_crash_smoke_failed",
                    error.to_string(),
                    Some("kind"),
                ),
                None,
            );
        }
    };

    let mut diagnostics = Vec::new();
    let readiness = recovery_readiness_label(smoke.snapshot.readiness);
    if let Some(expected) = &expectations.expected_readiness {
        if readiness != expected {
            diagnostics.push(diagnostic(
                EvalDiagnosticSeverity::Error,
                "recovery_readiness_mismatch",
                format!("expected readiness {expected}, got {readiness}"),
                Some("expectations.recovery.expected_readiness"),
            ));
        }
    }
    if smoke.snapshot.replay.replayed_event_count
        < expectations.min_replayed_event_count.unwrap_or(0)
    {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "recovery_replayed_event_count_too_low",
            format!(
                "expected at least {} replayed events, got {}",
                expectations.min_replayed_event_count.unwrap_or(0),
                smoke.snapshot.replay.replayed_event_count
            ),
            Some("expectations.recovery.min_replayed_event_count"),
        ));
    }
    if smoke.snapshot.idempotent_recovery.reusable_effect_count
        < expectations.min_idempotent_effect_count.unwrap_or(0)
    {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "recovery_idempotent_effect_count_too_low",
            format!(
                "expected at least {} reusable effects, got {}",
                expectations.min_idempotent_effect_count.unwrap_or(0),
                smoke.snapshot.idempotent_recovery.reusable_effect_count
            ),
            Some("expectations.recovery.min_idempotent_effect_count"),
        ));
    }
    if smoke.snapshot.idempotent_recovery.pending_tool_count
        < expectations.min_pending_tool_count.unwrap_or(0)
    {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "recovery_pending_tool_count_too_low",
            format!(
                "expected at least {} pending tools, got {}",
                expectations.min_pending_tool_count.unwrap_or(0),
                smoke.snapshot.idempotent_recovery.pending_tool_count
            ),
            Some("expectations.recovery.min_pending_tool_count"),
        ));
    }
    if (smoke.snapshot.conflicts.len() as u32) < expectations.min_conflict_count.unwrap_or(0) {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "recovery_conflict_count_too_low",
            format!(
                "expected at least {} conflicts, got {}",
                expectations.min_conflict_count.unwrap_or(0),
                smoke.snapshot.conflicts.len()
            ),
            Some("expectations.recovery.min_conflict_count"),
        ));
    }
    if expectations.requires_retry_plan
        && !matches!(
            smoke.retry_plan.readiness,
            agent_types::CommandRetryReadiness::Retryable
        )
    {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            "recovery_retry_plan_missing",
            "expected a retryable command retry plan",
            Some("expectations.recovery.requires_retry_plan"),
        ));
    }

    let status = if diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::RecoveryCrashSmoke,
            event_count: smoke.snapshot.replay.replayed_event_count as usize,
            checkpoint_count: usize::from(smoke.snapshot.checkpoint.is_some()),
            trace_entries: smoke.snapshot.conflicts.len(),
            audit_records: 0,
            telemetry_records: 0,
            replayable: !smoke.dry_run.blocked
                || smoke.snapshot.readiness == agent_types::RecoveryReadiness::HasConflict,
            diagnostics: diagnostics.len() + smoke.snapshot.diagnostics.len(),
        }),
        diagnostics,
    }
}

fn recovery_readiness_label(readiness: agent_types::RecoveryReadiness) -> &'static str {
    match readiness {
        agent_types::RecoveryReadiness::Ready => "ready",
        agent_types::RecoveryReadiness::NeedsApproval => "needs_approval",
        agent_types::RecoveryReadiness::PendingToolCalls => "pending_tool_calls",
        agent_types::RecoveryReadiness::HasConflict => "has_conflict",
        agent_types::RecoveryReadiness::CorruptCheckpoint => "corrupt_checkpoint",
        agent_types::RecoveryReadiness::StorageError => "storage_error",
        agent_types::RecoveryReadiness::Blocked => "blocked",
    }
}

fn evaluate_context_quality_workspace(
    workspace_root: &std::path::Path,
    expectations: &EvalContextQualityExpectations,
) -> Result<ContextQualityEvalOutcome, String> {
    let query = ContextRetrievalQuery::new(
        expectations.task.clone(),
        8,
        1_200,
        expectations.required_selected_paths.clone(),
    )
    .map_err(|error| error.to_string())?;

    if expectations.required_pinned_paths.is_empty() {
        let report = retrieve_project_context_quality(ProjectContextRetrievalInput {
            workspace_root: workspace_root.to_path_buf(),
            limits: context_quality_project_limits(),
            query,
        })
        .map_err(|error| format!("{} ({})", error, error.code()))?;
        return Ok(ContextQualityEvalOutcome {
            selected_paths: report
                .retrieval
                .selected
                .iter()
                .map(|candidate| candidate.path.clone())
                .collect(),
            pinned_paths: BTreeSet::new(),
            retrieval_included: None,
            signal_count: report.quality.signals.len(),
            trace_diagnostic_count: report.quality.diagnostics.len(),
        });
    }

    let pins = expectations
        .required_pinned_paths
        .iter()
        .map(|path| PinnedProjectContextPin::required(path.clone(), "eval required context"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let pinned_request = PinnedProjectContextRequest::new(
        pins,
        ContextBudget {
            max_tokens: 1_200,
            reserved_response_tokens: 0,
        },
        true,
    )
    .map_err(|error| error.to_string())?;
    let report = retrieve_project_context_pinned_quality(ProjectContextPinnedRetrievalInput {
        workspace_root: workspace_root.to_path_buf(),
        limits: context_quality_project_limits(),
        query,
        pinned_request,
    })
    .map_err(|error| format!("{} ({})", error, error.code()))?;

    Ok(ContextQualityEvalOutcome {
        selected_paths: report
            .retrieval
            .selected
            .iter()
            .map(|candidate| candidate.path.clone())
            .collect(),
        pinned_paths: report
            .pinned
            .included
            .iter()
            .map(|item| item.path.clone())
            .collect(),
        retrieval_included: Some(report.pinned.retrieval_included),
        signal_count: report.quality.signals.len(),
        trace_diagnostic_count: report.quality.diagnostics.len(),
    })
}

fn context_quality_project_limits() -> ProjectContextLimits {
    ProjectContextLimits::new(512, 8, 64 * 1024, 2 * 1024 * 1024)
        .expect("context quality eval limits are valid")
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
                format!("required event '{required}' was not present in scenario events"),
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
