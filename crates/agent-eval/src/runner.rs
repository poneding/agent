use std::collections::BTreeSet;
use std::fs;

use agent_core::{
    ContextBudget, ContextRetrievalQuery, ObservabilityEvidenceBundle,
    ObservabilityEvidenceOptions, PinnedProjectContextPin, PinnedProjectContextRequest,
    ProjectContextLimits, ProjectContextPinnedRetrievalInput, ProjectContextRetrievalInput,
    RuntimeMemoryConfig, RuntimeMemoryInput, RuntimeMemoryProposalInput,
    RuntimeMemoryProposalPolicy, assemble_runtime_memory, build_observability_evidence,
    replay_final_review, retrieve_project_context_pinned_quality, retrieve_project_context_quality,
    run_local_coding_fixture, run_recovery_crash_smoke_fixture,
    verify_observability_replay_fixture,
};
use agent_memory::{
    InMemoryMemoryEditor, InMemoryMemoryProposalStore, MemoryEditor, MemoryImportPolicy,
    MemoryProposalDecision, MemoryProposalStore, MemoryRecord, MemoryRetentionPolicy,
    MemoryRetrievalCandidate, MemoryScope, MemorySensitivity, MemorySource, MemoryWriteProposal,
};
use agent_policy::{CapabilityAction, ResourceKind, ResourceScope};
use agent_storage::{check_replay_compatibility, load_replay_fixture_json};
use agent_team::{
    BlackboardEntryKind, CapabilityGrant, CapabilitySet, HandoffStatus, ReviewVote, SubagentRole,
    TeamBlackboardEntrySummary, TeamBlackboardSummary, TeamCapabilityCeilingSummary,
    TeamHandoffSummary, TeamMergeBlockerKind, TeamMergeReadinessReport, TeamMergeReviewSummary,
    TeamWorkflowReadiness, TeamWorkflowSnapshot, TeamWorktreeState, TeamWorktreeSummary,
};
use agent_types::{
    AgentId, ConversationId, EventId, RecoveryConflictKind, RecoveryConflictNextAction,
    RecoveryConflictSide, RecoveryConflictSummary, TaskId, TeamId, WorkspaceId,
};
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
        EvalScenarioKind::TeamWorkflowSmoke => run_team_workflow_smoke_scenario(scenario),
        EvalScenarioKind::MemorySmoke => run_memory_smoke_scenario(scenario),
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemorySmokeEvalOutcome {
    record_count: u32,
    proposal_count: u32,
    proposal_decision_count: u32,
    selected_count: u32,
    rejected_count: u32,
    tombstone_count: u32,
    export_record_count: u32,
    export_sensitive_record_count: u32,
    import_accepted_count: u32,
    rebuild_indexed_records: u32,
    rebuild_skipped_tombstoned: u32,
    rebuild_stale_removed: u32,
    disabled_diagnostic: bool,
    diagnostic_count: usize,
    redacted: bool,
    leaked_secret: bool,
}

fn run_memory_smoke_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(expectations) = scenario.manifest.expectations.memory.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "memory_expectations_missing",
                "memory expectations are required",
                Some("expectations.memory"),
            ),
            None,
        );
    };

    let outcome = match build_memory_smoke() {
        Ok(outcome) => outcome,
        Err(message) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "memory_smoke_failed",
                    message,
                    Some("kind"),
                ),
                None,
            );
        }
    };

    let mut diagnostics = Vec::new();
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.record_count,
        expectations.min_record_count,
        "memory_record_count_too_low",
        "records",
        "expectations.memory.min_record_count",
    );
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.proposal_count,
        expectations.min_proposal_count,
        "memory_proposal_count_too_low",
        "proposals",
        "expectations.memory.min_proposal_count",
    );
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.selected_count,
        expectations.min_selected_count,
        "memory_selected_count_too_low",
        "selected memories",
        "expectations.memory.min_selected_count",
    );
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.tombstone_count,
        expectations.min_tombstone_count,
        "memory_tombstone_count_too_low",
        "tombstones",
        "expectations.memory.min_tombstone_count",
    );
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.import_accepted_count,
        expectations.min_import_accepted_count,
        "memory_import_accepted_count_too_low",
        "accepted imports",
        "expectations.memory.min_import_accepted_count",
    );
    push_memory_min_diagnostic(
        &mut diagnostics,
        outcome.rebuild_stale_removed,
        expectations.min_rebuild_stale_removed_count,
        "memory_rebuild_stale_removed_count_too_low",
        "stale rebuild removals",
        "expectations.memory.min_rebuild_stale_removed_count",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        expectations.require_proposal_decisions && outcome.proposal_decision_count == 0,
        "memory_proposal_decisions_missing",
        "expected proposal approve/reject/commit evidence",
        "expectations.memory.require_proposal_decisions",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        expectations.require_import_export
            && (outcome.export_record_count == 0 || outcome.import_accepted_count == 0),
        "memory_import_export_missing",
        "expected import/export evidence",
        "expectations.memory.require_import_export",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        outcome.export_sensitive_record_count != 0,
        "memory_sensitive_exported_by_default",
        "default memory export should exclude sensitive records",
        "memory_smoke.export.sensitive_record_count",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        expectations.require_rebuild
            && (outcome.rebuild_indexed_records == 0
                || outcome.rebuild_skipped_tombstoned == 0
                || outcome.rebuild_stale_removed == 0),
        "memory_rebuild_missing",
        "expected rebuild indexed, skipped tombstone, and stale removal evidence",
        "expectations.memory.require_rebuild",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        expectations.require_disabled_diagnostic && !outcome.disabled_diagnostic,
        "memory_disabled_diagnostic_missing",
        "expected disabled-memory diagnostic evidence",
        "expectations.memory.require_disabled_diagnostic",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        expectations.require_redaction && !outcome.redacted,
        "memory_redaction_marker_missing",
        "expected redacted marker in memory smoke evidence",
        "expectations.memory.require_redaction",
    );
    push_memory_bool_diagnostic(
        &mut diagnostics,
        outcome.leaked_secret,
        "memory_secret_leaked",
        "memory smoke evidence leaked raw secret-like text",
        "memory_smoke.serialized",
    );

    let status = if diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::MemorySmoke,
            event_count: outcome.record_count as usize,
            checkpoint_count: outcome.tombstone_count as usize,
            trace_entries: (outcome.selected_count + outcome.rejected_count) as usize,
            audit_records: outcome.proposal_count as usize,
            telemetry_records: (outcome.export_record_count + outcome.import_accepted_count)
                as usize,
            replayable: true,
            diagnostics: diagnostics.len() + outcome.diagnostic_count,
        }),
        diagnostics,
    }
}

fn push_memory_min_diagnostic(
    diagnostics: &mut Vec<EvalDiagnostic>,
    actual: u32,
    expected: Option<u32>,
    code: &'static str,
    label: &'static str,
    field: &'static str,
) {
    let Some(expected) = expected else {
        return;
    };
    if actual < expected {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            code,
            format!("expected at least {expected} {label}, got {actual}"),
            Some(field),
        ));
    }
}

fn push_memory_bool_diagnostic(
    diagnostics: &mut Vec<EvalDiagnostic>,
    condition: bool,
    code: &'static str,
    message: &'static str,
    field: &'static str,
) {
    if condition {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            code,
            message,
            Some(field),
        ));
    }
}

fn build_memory_smoke() -> Result<MemorySmokeEvalOutcome, String> {
    let public = memory_smoke_record(
        "memory-public",
        "prefer deterministic memory smoke tests",
        MemorySensitivity::Public,
    );
    let internal = memory_smoke_record(
        "memory-internal",
        "project uses memory review before durable writes",
        MemorySensitivity::Internal,
    );
    let sensitive = memory_smoke_record(
        "memory-sensitive",
        "private api_key sk-live-secret-token password should stay hidden",
        MemorySensitivity::Sensitive,
    );

    let runtime = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory smoke review preference".to_string(),
        candidates: vec![
            memory_smoke_candidate(public.clone(), 0.8, Some(0.4), 0.95),
            memory_smoke_candidate(sensitive.clone(), 0.4, Some(0.2), 0.60),
        ],
        config: RuntimeMemoryConfig {
            max_selected: 1,
            ..Default::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: vec![RuntimeMemoryProposalInput {
            proposal_id: "proposal-runtime".to_string(),
            proposed_record_id: "memory-runtime-proposal".to_string(),
            scope: memory_smoke_conversation_scope(),
            content: "remember raw sk-live-secret-token only through review".to_string(),
            summary: "remember memory review preference with sk-live-secret-token".to_string(),
            source_event_ids: vec![memory_smoke_event_id()],
            confidence: 87,
            sensitivity: MemorySensitivity::Sensitive,
        }],
    })
    .map_err(|error| error.to_string())?
    .evidence;

    let disabled = assemble_runtime_memory(RuntimeMemoryInput {
        query: "memory disabled".to_string(),
        candidates: Vec::new(),
        config: RuntimeMemoryConfig {
            enabled: false,
            ..Default::default()
        },
        proposal_policy: RuntimeMemoryProposalPolicy::default(),
        proposal_inputs: Vec::new(),
    })
    .map_err(|error| error.to_string())?
    .evidence;

    let mut proposals = InMemoryMemoryProposalStore::default();
    let approve = MemoryWriteProposal::new(
        "proposal-commit",
        "memory-committed",
        memory_smoke_project_scope(),
        "prefer memory closeout docs",
        "remember memory closeout docs",
        vec![memory_smoke_event_id()],
        91,
        MemorySensitivity::Public,
    )
    .map_err(|error| error.to_string())?;
    let reject = MemoryWriteProposal::new(
        "proposal-reject",
        "memory-rejected",
        memory_smoke_conversation_scope(),
        "do not store sk-live-secret-token values",
        "reject memory with sk-live-secret-token",
        vec![memory_smoke_event_id()],
        60,
        MemorySensitivity::Sensitive,
    )
    .map_err(|error| error.to_string())?;
    proposals
        .add_proposal(approve)
        .map_err(|error| error.to_string())?;
    proposals
        .add_proposal(reject)
        .map_err(|error| error.to_string())?;
    proposals
        .resolve("proposal-commit", MemoryProposalDecision::Approve)
        .map_err(|error| error.to_string())?;
    let committed = proposals
        .commit("proposal-commit")
        .map_err(|error| error.to_string())?;
    proposals
        .resolve(
            "proposal-reject",
            MemoryProposalDecision::Reject {
                reason: "contains sk-live-secret-token".to_string(),
            },
        )
        .map_err(|error| error.to_string())?;
    let proposal_summaries = proposals
        .list_proposals()
        .iter()
        .map(MemoryWriteProposal::safe_summary)
        .collect::<Vec<_>>();
    let proposal_decision_count = proposal_summaries
        .iter()
        .filter(|summary| {
            matches!(
                summary.status,
                agent_memory::MemoryProposalStatus::Committed
                    | agent_memory::MemoryProposalStatus::Rejected
            )
        })
        .count();

    let mut editor = InMemoryMemoryEditor::default();
    editor
        .add_record(public)
        .map_err(|error| error.to_string())?;
    editor
        .add_record(internal)
        .map_err(|error| error.to_string())?;
    editor
        .add_record(sensitive)
        .map_err(|error| error.to_string())?;
    editor
        .add_record(committed)
        .map_err(|error| error.to_string())?;
    editor
        .upsert_embedding_marker("memory-public")
        .map_err(|error| error.to_string())?;
    editor
        .upsert_embedding_marker("memory-sensitive")
        .map_err(|error| error.to_string())?;
    editor
        .delete_memory(
            &memory_smoke_project_scope(),
            "memory-sensitive",
            "cleanup sk-live-secret-token password",
            Some(memory_smoke_event_id()),
        )
        .map_err(|error| error.to_string())?;
    let tombstones = editor.tombstones();
    let rebuild = editor.rebuild_index().map_err(|error| error.to_string())?;
    let archive = editor
        .export_archive(MemoryRetentionPolicy {
            include_sensitive: false,
            max_records: None,
        })
        .map_err(|error| error.to_string())?;
    let mut imported = InMemoryMemoryEditor::default();
    let import = imported
        .import_archive(&archive, MemoryImportPolicy::default())
        .map_err(|error| error.to_string())?;

    let serialized = serde_json::to_string(&(
        &runtime,
        &disabled,
        &proposal_summaries,
        &archive,
        &import.report,
        &tombstones,
        &rebuild,
    ))
    .map_err(|error| format!("failed to serialize memory smoke: {error}"))?;
    let leaked_secret = [
        "sk-live-secret-token",
        "api_key",
        "password should stay hidden",
        "remember raw",
        "do not store sk-live",
    ]
    .into_iter()
    .any(|needle| serialized.contains(needle));

    Ok(MemorySmokeEvalOutcome {
        record_count: runtime
            .snapshot
            .records
            .len()
            .try_into()
            .unwrap_or(u32::MAX),
        proposal_count: (runtime.proposals.created_count + proposal_summaries.len())
            .try_into()
            .unwrap_or(u32::MAX),
        proposal_decision_count: proposal_decision_count.try_into().unwrap_or(u32::MAX),
        selected_count: runtime
            .influence
            .selected_count
            .try_into()
            .unwrap_or(u32::MAX),
        rejected_count: runtime
            .influence
            .rejected_count
            .try_into()
            .unwrap_or(u32::MAX),
        tombstone_count: tombstones.len().try_into().unwrap_or(u32::MAX),
        export_record_count: archive.manifest.record_count.try_into().unwrap_or(u32::MAX),
        export_sensitive_record_count: archive
            .manifest
            .sensitive_record_count
            .try_into()
            .unwrap_or(u32::MAX),
        import_accepted_count: import.report.accepted.try_into().unwrap_or(u32::MAX),
        rebuild_indexed_records: rebuild.indexed_records.try_into().unwrap_or(u32::MAX),
        rebuild_skipped_tombstoned: rebuild.skipped_tombstoned.try_into().unwrap_or(u32::MAX),
        rebuild_stale_removed: rebuild
            .stale_embeddings_removed
            .try_into()
            .unwrap_or(u32::MAX),
        disabled_diagnostic: disabled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "memory_disabled"),
        diagnostic_count: runtime.diagnostics.len() + disabled.diagnostics.len(),
        redacted: serialized.contains("[redacted]"),
        leaked_secret,
    })
}

fn memory_smoke_record(id: &str, text: &str, sensitivity: MemorySensitivity) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        scope: memory_smoke_project_scope(),
        text: text.to_string(),
        source: MemorySource {
            event_id: Some(memory_smoke_event_id()),
            description: "memory smoke fixture".to_string(),
        },
        confidence: 91,
        sensitivity,
    }
}

fn memory_smoke_candidate(
    record: MemoryRecord,
    lexical_score: f32,
    semantic_score: Option<f32>,
    combined_score: f32,
) -> MemoryRetrievalCandidate {
    MemoryRetrievalCandidate {
        record,
        lexical_score,
        semantic_score,
        combined_score,
        matched_terms: vec!["memory".to_string(), "sk-live-secret-token".to_string()],
    }
}

fn memory_smoke_project_scope() -> MemoryScope {
    MemoryScope::Project {
        workspace_id: memory_smoke_workspace_id(),
    }
}

fn memory_smoke_conversation_scope() -> MemoryScope {
    MemoryScope::Conversation(memory_smoke_conversation_id())
}

fn memory_smoke_workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000641"
        .parse()
        .expect("valid workspace id")
}

fn memory_smoke_conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000000642"
        .parse()
        .expect("valid conversation id")
}

fn memory_smoke_event_id() -> EventId {
    "00000000-0000-0000-0000-000000000643"
        .parse()
        .expect("valid event id")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TeamWorkflowEvalOutcome {
    snapshot_count: u32,
    worktree_count: u32,
    handoff_count: u32,
    blackboard_entry_count: u32,
    conflict_count: u32,
    merge_blocker_count: u32,
    diagnostic_count: usize,
    capability_reduced: bool,
    policy_blocked: bool,
    conflict_blocker: bool,
    review_blocker: bool,
    has_merge_ready: bool,
    has_conflict_ready: bool,
    has_policy_blocked: bool,
    has_review_blocked: bool,
    redacted: bool,
    leaked_secret: bool,
}

fn run_team_workflow_smoke_scenario(scenario: &ValidatedEvalScenario) -> EvalScenarioReport {
    let Some(expectations) = scenario.manifest.expectations.team_workflow.as_ref() else {
        return failed_report(
            scenario,
            diagnostic(
                EvalDiagnosticSeverity::Error,
                "team_workflow_expectations_missing",
                "team workflow expectations are required",
                Some("expectations.team_workflow"),
            ),
            None,
        );
    };

    let outcome = match build_team_workflow_smoke() {
        Ok(outcome) => outcome,
        Err(message) => {
            return failed_report(
                scenario,
                diagnostic(
                    EvalDiagnosticSeverity::Error,
                    "team_workflow_smoke_failed",
                    message,
                    Some("kind"),
                ),
                None,
            );
        }
    };

    let mut diagnostics = Vec::new();
    push_team_min_diagnostic(
        &mut diagnostics,
        outcome.worktree_count,
        expectations.min_worktree_count,
        "team_workflow_worktree_count_too_low",
        "worktrees",
        "expectations.team_workflow.min_worktree_count",
    );
    push_team_min_diagnostic(
        &mut diagnostics,
        outcome.handoff_count,
        expectations.min_handoff_count,
        "team_workflow_handoff_count_too_low",
        "handoffs",
        "expectations.team_workflow.min_handoff_count",
    );
    push_team_min_diagnostic(
        &mut diagnostics,
        outcome.blackboard_entry_count,
        expectations.min_blackboard_entry_count,
        "team_workflow_blackboard_entry_count_too_low",
        "blackboard entries",
        "expectations.team_workflow.min_blackboard_entry_count",
    );
    push_team_min_diagnostic(
        &mut diagnostics,
        outcome.conflict_count,
        expectations.min_conflict_count,
        "team_workflow_conflict_count_too_low",
        "conflicts",
        "expectations.team_workflow.min_conflict_count",
    );
    push_team_min_diagnostic(
        &mut diagnostics,
        outcome.merge_blocker_count,
        expectations.min_merge_blocker_count,
        "team_workflow_merge_blocker_count_too_low",
        "merge blockers",
        "expectations.team_workflow.min_merge_blocker_count",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        expectations.require_capability_reduction && !outcome.capability_reduced,
        "team_workflow_capability_not_reduced",
        "expected reduced capability ceiling evidence",
        "expectations.team_workflow.require_capability_reduction",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        expectations.require_policy_blocked && !outcome.policy_blocked,
        "team_workflow_policy_block_missing",
        "expected policy-blocked merge readiness evidence",
        "expectations.team_workflow.require_policy_blocked",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        expectations.require_conflict_blocker && !outcome.conflict_blocker,
        "team_workflow_conflict_blocker_missing",
        "expected conflict merge blocker evidence",
        "expectations.team_workflow.require_conflict_blocker",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        expectations.require_review_blocker && !outcome.review_blocker,
        "team_workflow_review_blocker_missing",
        "expected review-blocking merge blocker evidence",
        "expectations.team_workflow.require_review_blocker",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        expectations.require_redaction && !outcome.redacted,
        "team_workflow_redaction_marker_missing",
        "expected redacted marker in team workflow evidence",
        "expectations.team_workflow.require_redaction",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        outcome.leaked_secret,
        "team_workflow_secret_leaked",
        "team workflow smoke evidence leaked secret-like text",
        "team_workflow_smoke.serialized",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        !outcome.has_merge_ready,
        "team_workflow_merge_ready_missing",
        "expected merge-ready workflow evidence",
        "team_workflow_smoke.readiness",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        !outcome.has_conflict_ready,
        "team_workflow_conflict_readiness_missing",
        "expected conflict readiness evidence",
        "team_workflow_smoke.readiness",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        !outcome.has_policy_blocked,
        "team_workflow_policy_readiness_missing",
        "expected policy-blocked readiness evidence",
        "team_workflow_smoke.readiness",
    );
    push_team_bool_diagnostic(
        &mut diagnostics,
        !outcome.has_review_blocked,
        "team_workflow_review_readiness_missing",
        "expected review-blocked readiness evidence",
        "team_workflow_smoke.readiness",
    );

    let status = if diagnostics.is_empty() {
        EvalScenarioStatus::Passed
    } else {
        EvalScenarioStatus::Failed
    };
    EvalScenarioReport {
        scenario_id: scenario.manifest.id.clone(),
        status,
        evidence_summary: Some(EvalEvidenceSummary {
            source: EvalEvidenceSource::TeamWorkflowSmoke,
            event_count: outcome.snapshot_count as usize,
            checkpoint_count: outcome.worktree_count as usize,
            trace_entries: outcome.blackboard_entry_count as usize,
            audit_records: outcome.merge_blocker_count as usize,
            telemetry_records: outcome.handoff_count as usize,
            replayable: true,
            diagnostics: diagnostics.len() + outcome.diagnostic_count,
        }),
        diagnostics,
    }
}

fn push_team_min_diagnostic(
    diagnostics: &mut Vec<EvalDiagnostic>,
    actual: u32,
    expected: Option<u32>,
    code: &'static str,
    label: &'static str,
    field: &'static str,
) {
    let Some(expected) = expected else {
        return;
    };
    if actual < expected {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            code,
            format!("expected at least {expected} {label}, got {actual}"),
            Some(field),
        ));
    }
}

fn push_team_bool_diagnostic(
    diagnostics: &mut Vec<EvalDiagnostic>,
    condition: bool,
    code: &'static str,
    message: &'static str,
    field: &'static str,
) {
    if condition {
        diagnostics.push(diagnostic(
            EvalDiagnosticSeverity::Error,
            code,
            message,
            Some(field),
        ));
    }
}

fn build_team_workflow_smoke() -> Result<TeamWorkflowEvalOutcome, String> {
    let reduced = reduced_capability_ceiling()?;
    let reduced_parent_count = 3_u32;
    let ready = team_smoke_snapshot(
        HandoffStatus::Completed,
        Some(ReviewVote::Approve),
        TeamMergeReviewSummary::new(1, 1, 0, 1).map_err(|error| error.to_string())?,
        reduced.clone(),
        0,
        Vec::new(),
    )?;
    let conflict = team_smoke_snapshot(
        HandoffStatus::Completed,
        Some(ReviewVote::Approve),
        TeamMergeReviewSummary::new(1, 1, 0, 1).map_err(|error| error.to_string())?,
        reduced.clone(),
        0,
        vec![team_smoke_conflict()],
    )?;
    let policy = team_smoke_snapshot(
        HandoffStatus::Completed,
        Some(ReviewVote::Approve),
        TeamMergeReviewSummary::new(1, 1, 0, 1).map_err(|error| error.to_string())?,
        TeamCapabilityCeilingSummary {
            grant_count: 0,
            grants: Vec::new(),
            policy_blocked: true,
        },
        0,
        Vec::new(),
    )?;
    let review_blocked = team_smoke_snapshot(
        HandoffStatus::Completed,
        Some(ReviewVote::Block),
        TeamMergeReviewSummary::new(1, 0, 1, 1).map_err(|error| error.to_string())?,
        reduced.clone(),
        0,
        Vec::new(),
    )?;

    let snapshots = vec![ready, conflict, policy, review_blocked];
    let reports = snapshots
        .iter()
        .map(TeamMergeReadinessReport::from_snapshot)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let serialized = serde_json::to_string(&(&snapshots, &reports))
        .map_err(|error| format!("failed to serialize team workflow smoke: {error}"))?;
    let lowered = serialized.to_ascii_lowercase();
    let leaked_secret = ["api_key", "secret", "token", "password", "sk-live"]
        .into_iter()
        .any(|needle| lowered.contains(needle));

    Ok(TeamWorkflowEvalOutcome {
        snapshot_count: snapshots.len() as u32,
        worktree_count: snapshots
            .iter()
            .map(|snapshot| snapshot.worktrees.len() as u32)
            .sum(),
        handoff_count: snapshots
            .iter()
            .map(|snapshot| snapshot.handoffs.len() as u32)
            .sum(),
        blackboard_entry_count: snapshots
            .iter()
            .map(|snapshot| snapshot.blackboard.entry_count)
            .sum(),
        conflict_count: snapshots
            .iter()
            .map(|snapshot| snapshot.conflicts.len() as u32)
            .sum(),
        merge_blocker_count: reports
            .iter()
            .map(|report| report.blockers.len() as u32)
            .sum(),
        diagnostic_count: snapshots
            .iter()
            .map(|snapshot| snapshot.diagnostics.len())
            .sum::<usize>()
            + reports
                .iter()
                .map(|report| report.diagnostics.len())
                .sum::<usize>(),
        capability_reduced: reduced.grant_count > 0 && reduced.grant_count < reduced_parent_count,
        policy_blocked: reports.iter().any(|report| {
            report
                .blockers
                .iter()
                .any(|blocker| blocker.kind == TeamMergeBlockerKind::PolicyBlocked)
        }),
        conflict_blocker: reports.iter().any(|report| {
            report
                .blockers
                .iter()
                .any(|blocker| blocker.kind == TeamMergeBlockerKind::Conflict)
        }),
        review_blocker: reports.iter().any(|report| {
            report
                .blockers
                .iter()
                .any(|blocker| blocker.kind == TeamMergeBlockerKind::BlockingReview)
        }),
        has_merge_ready: snapshots
            .iter()
            .any(|snapshot| snapshot.readiness == TeamWorkflowReadiness::MergeReady),
        has_conflict_ready: snapshots
            .iter()
            .any(|snapshot| snapshot.readiness == TeamWorkflowReadiness::HasConflict),
        has_policy_blocked: snapshots
            .iter()
            .any(|snapshot| snapshot.readiness == TeamWorkflowReadiness::PolicyBlocked),
        has_review_blocked: snapshots
            .iter()
            .any(|snapshot| snapshot.readiness == TeamWorkflowReadiness::Blocked),
        redacted: serialized.contains("[redacted]"),
        leaked_secret,
    })
}

fn team_smoke_snapshot(
    handoff_status: HandoffStatus,
    review_vote: Option<ReviewVote>,
    merge_review: TeamMergeReviewSummary,
    capability_ceiling: TeamCapabilityCeilingSummary,
    pending_approval_count: u32,
    conflicts: Vec<RecoveryConflictSummary>,
) -> Result<TeamWorkflowSnapshot, String> {
    TeamWorkflowSnapshot::new_with_pending_approvals(
        team_smoke_team_id(),
        team_smoke_lead_agent_id(),
        team_smoke_workspace_id(),
        team_smoke_task_id(),
        vec![
            TeamWorktreeSummary::new(
                "worktree-team-smoke",
                "main",
                "agent/m63-team-smoke",
                ".worktrees/agent-m63-team-smoke",
                TeamWorktreeState::Active,
            )
            .map_err(|error| error.to_string())?,
        ],
        vec![
            TeamHandoffSummary::new(
                team_smoke_subagent_id(),
                SubagentRole::Implementer,
                "worktree-team-smoke",
                handoff_status,
                "Implement team workflow smoke",
            )
            .map_err(|error| error.to_string())?,
        ],
        team_smoke_blackboard(review_vote)?,
        merge_review,
        capability_ceiling,
        pending_approval_count,
        conflicts,
        Vec::new(),
    )
    .map_err(|error| error.to_string())
}

fn team_smoke_blackboard(review_vote: Option<ReviewVote>) -> Result<TeamBlackboardSummary, String> {
    TeamBlackboardSummary::new(vec![
        TeamBlackboardEntrySummary::new(
            BlackboardEntryKind::Finding,
            "Subagent produced bounded worktree evidence",
            1,
            1,
            None,
        )
        .map_err(|error| error.to_string())?,
        TeamBlackboardEntrySummary::new(
            BlackboardEntryKind::Review,
            "Reviewer checked api_key sk-live-secret-token password evidence",
            1,
            2,
            review_vote,
        )
        .map_err(|error| error.to_string())?,
    ])
    .map_err(|error| error.to_string())
}

fn reduced_capability_ceiling() -> Result<TeamCapabilityCeilingSummary, String> {
    let parent = capability_set(vec![
        capability_grant(
            CapabilityAction::Read,
            ResourceKind::File,
            "crates/agent-team/**",
        )?,
        capability_grant(CapabilityAction::Search, ResourceKind::Workspace, ".")?,
        capability_grant(
            CapabilityAction::Execute,
            ResourceKind::Command,
            "cargo test -p agent-team",
        )?,
    ])?;
    let role = capability_set(vec![
        capability_grant(
            CapabilityAction::Read,
            ResourceKind::File,
            "crates/agent-team/**",
        )?,
        capability_grant(CapabilityAction::Search, ResourceKind::Workspace, ".")?,
    ])?;
    let task = capability_set(vec![capability_grant(
        CapabilityAction::Read,
        ResourceKind::File,
        "crates/agent-team/**",
    )?])?;

    Ok(TeamCapabilityCeilingSummary::from_reduced_sets(
        &parent, &role, &task, false,
    ))
}

fn capability_grant(
    action: CapabilityAction,
    kind: ResourceKind,
    pattern: &str,
) -> Result<CapabilityGrant, String> {
    let scope = ResourceScope::new(kind, pattern).map_err(|error| error.to_string())?;
    CapabilityGrant::new(action, scope).map_err(|error| error.to_string())
}

fn capability_set(grants: Vec<CapabilityGrant>) -> Result<CapabilitySet, String> {
    CapabilitySet::new(grants).map_err(|error| error.to_string())
}

fn team_smoke_conflict() -> RecoveryConflictSummary {
    RecoveryConflictSummary {
        path: "crates/agent-team/src/workflow.rs".to_string(),
        kind: RecoveryConflictKind::BothModified,
        lead_change: RecoveryConflictSide {
            source: "lead".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("lead-hash".to_string()),
        },
        incoming_change: RecoveryConflictSide {
            source: "subagent".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("incoming-hash".to_string()),
        },
        next_action: RecoveryConflictNextAction::InspectDiff,
        diagnostics: Vec::new(),
    }
}

fn team_smoke_team_id() -> TeamId {
    "00000000-0000-0000-0000-000000000263"
        .parse()
        .expect("valid team id")
}

fn team_smoke_lead_agent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000264"
        .parse()
        .expect("valid agent id")
}

fn team_smoke_subagent_id() -> AgentId {
    "00000000-0000-0000-0000-000000000265"
        .parse()
        .expect("valid agent id")
}

fn team_smoke_workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000000266"
        .parse()
        .expect("valid workspace id")
}

fn team_smoke_task_id() -> TaskId {
    "00000000-0000-0000-0000-000000000267"
        .parse()
        .expect("valid task id")
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
