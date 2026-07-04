use agent_storage::{EventStore, InMemoryEventStore, ReplayCompatibilityFixture};
use agent_telemetry::{
    TelemetryExportBundle, TelemetryExportError, TelemetryExportOptions, TelemetryRecord,
    export_telemetry_bundle_at,
};
use agent_types::{ConversationId, RuntimeEvent, RuntimeEventPayload, Timestamp};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AuditExportBundle, AuditExportError, AuditExportOptions, TraceEntry, TraceError, TraceQuery,
    build_trace, export_audit_bundle_at,
};

pub const OBSERVABILITY_EVIDENCE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservabilityEvidenceOptions {
    pub conversation_id: Option<ConversationId>,
    pub include_trace: bool,
    pub include_audit: bool,
    pub include_telemetry: bool,
    pub include_replay: bool,
    pub max_payload_bytes: usize,
    pub telemetry: TelemetryExportOptions,
}

impl Default for ObservabilityEvidenceOptions {
    fn default() -> Self {
        Self {
            conversation_id: None,
            include_trace: true,
            include_audit: true,
            include_telemetry: true,
            include_replay: true,
            max_payload_bytes: 4096,
            telemetry: TelemetryExportOptions::default(),
        }
    }
}

impl ObservabilityEvidenceOptions {
    fn validate(self) -> Result<Self, ObservabilityEvidenceError> {
        if self.max_payload_bytes == 0 {
            return Err(ObservabilityEvidenceError::InvalidMaxPayloadBytes);
        }
        if !self.include_trace
            && !self.include_audit
            && !self.include_telemetry
            && !self.include_replay
        {
            return Err(ObservabilityEvidenceError::NoEvidenceSectionsSelected);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityEvidenceBundle {
    pub schema_version: u32,
    pub generated_at: Timestamp,
    pub summary: ObservabilityEvidenceSummary,
    pub trace: Vec<TraceEntry>,
    pub audit: AuditExportBundle,
    pub telemetry: TelemetryExportBundle,
    pub replay: ReplayEvidenceSummary,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityEvidenceSummary {
    pub total_events: usize,
    pub trace_entries: usize,
    pub audit_records: usize,
    pub telemetry_records: usize,
    pub replayable: bool,
    pub diagnostics: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayEvidenceSummary {
    pub replayable: bool,
    pub event_count: usize,
    pub checkpoint_count: usize,
    pub replay_started_count: usize,
    pub replay_completed_count: usize,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EvidenceReplayVerificationExpectation {
    pub event_count: Option<usize>,
    pub checkpoint_count: Option<usize>,
    pub audit_record_count: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceReplayVerificationReport {
    pub replayable: bool,
    pub source_event_count: usize,
    pub source_checkpoint_count: usize,
    pub regenerated_trace_count: usize,
    pub regenerated_audit_count: usize,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ObservabilityEvidenceError {
    #[error("observability evidence max_payload_bytes must be greater than zero")]
    InvalidMaxPayloadBytes,
    #[error("observability evidence must include at least one section")]
    NoEvidenceSectionsSelected,
    #[error("trace evidence failed: {0}")]
    Trace(String),
    #[error("audit evidence failed: {0}")]
    Audit(String),
    #[error("telemetry evidence failed: {0}")]
    Telemetry(String),
}

pub fn build_observability_evidence(
    events: &[RuntimeEvent],
    telemetry: &[TelemetryRecord],
    options: ObservabilityEvidenceOptions,
) -> Result<ObservabilityEvidenceBundle, ObservabilityEvidenceError> {
    build_observability_evidence_at(events, telemetry, options, Timestamp::now_utc())
}

pub fn build_observability_evidence_at(
    events: &[RuntimeEvent],
    telemetry: &[TelemetryRecord],
    options: ObservabilityEvidenceOptions,
    generated_at: Timestamp,
) -> Result<ObservabilityEvidenceBundle, ObservabilityEvidenceError> {
    let options = options.validate()?;
    let selection = select_events(events, options.conversation_id);

    let trace = if options.include_trace {
        build_trace_evidence(selection.conversation_id, &selection.events)?
    } else {
        Vec::new()
    };
    let audit = if options.include_audit {
        export_audit_bundle_at(
            &selection.events,
            AuditExportOptions {
                include_diagnostics: true,
                max_payload_bytes: options.max_payload_bytes,
            },
            generated_at,
        )
        .map_err(audit_error)?
    } else {
        export_audit_bundle_at(&[], AuditExportOptions::default(), generated_at)
            .map_err(audit_error)?
    };
    let telemetry = if options.include_telemetry {
        export_telemetry_bundle_at(telemetry, options.telemetry, generated_at)
            .map_err(telemetry_error)?
    } else {
        export_telemetry_bundle_at(&[], TelemetryExportOptions::default(), generated_at)
            .map_err(telemetry_error)?
    };
    let replay = if options.include_replay {
        build_replay_summary(&selection.events)
    } else {
        ReplayEvidenceSummary::default()
    };

    let mut diagnostics = selection.diagnostics;
    diagnostics.extend(replay.diagnostics.iter().cloned());
    let summary = ObservabilityEvidenceSummary {
        total_events: selection.events.len(),
        trace_entries: trace.len(),
        audit_records: audit.summary.audit_records,
        telemetry_records: telemetry.summary.exported_records,
        replayable: replay.replayable,
        diagnostics: diagnostics.len(),
    };

    Ok(ObservabilityEvidenceBundle {
        schema_version: OBSERVABILITY_EVIDENCE_SCHEMA_VERSION,
        generated_at,
        summary,
        trace,
        audit,
        telemetry,
        replay,
        diagnostics,
    })
}

pub fn verify_observability_replay_fixture(
    fixture: &ReplayCompatibilityFixture,
) -> Result<EvidenceReplayVerificationReport, ObservabilityEvidenceError> {
    verify_observability_evidence_replay(
        &fixture.events,
        EvidenceReplayVerificationExpectation {
            event_count: Some(fixture.expected.event_count),
            checkpoint_count: Some(fixture.expected.checkpoint_count),
            audit_record_count: Some(fixture.expected.event_count),
        },
    )
}

pub fn verify_observability_evidence_replay(
    events: &[RuntimeEvent],
    expectation: EvidenceReplayVerificationExpectation,
) -> Result<EvidenceReplayVerificationReport, ObservabilityEvidenceError> {
    let evidence = build_observability_evidence(
        events,
        &[],
        ObservabilityEvidenceOptions {
            include_telemetry: false,
            ..ObservabilityEvidenceOptions::default()
        },
    )?;
    let source_checkpoint_count = count_checkpoint_events(events);
    let mut diagnostics = evidence.diagnostics.clone();
    let mut mismatch = false;

    if let Some(expected) = expectation.event_count {
        if expected != events.len() {
            mismatch = true;
            diagnostics.push(format!(
                "event count mismatch: expected {expected}, regenerated {}",
                events.len()
            ));
        }
    }

    if let Some(expected) = expectation.checkpoint_count {
        if expected != source_checkpoint_count {
            mismatch = true;
            diagnostics.push(format!(
                "checkpoint count mismatch: expected {expected}, regenerated {source_checkpoint_count}"
            ));
        }
    }

    if let Some(expected) = expectation.audit_record_count {
        if expected != evidence.summary.audit_records {
            mismatch = true;
            diagnostics.push(format!(
                "audit record count mismatch: expected {expected}, regenerated {}",
                evidence.summary.audit_records
            ));
        }
    }

    Ok(EvidenceReplayVerificationReport {
        replayable: evidence.replay.replayable && !mismatch,
        source_event_count: events.len(),
        source_checkpoint_count,
        regenerated_trace_count: evidence.summary.trace_entries,
        regenerated_audit_count: evidence.summary.audit_records,
        diagnostics,
    })
}

struct EventSelection {
    conversation_id: Option<ConversationId>,
    events: Vec<RuntimeEvent>,
    diagnostics: Vec<String>,
}

fn select_events(
    events: &[RuntimeEvent],
    requested_conversation_id: Option<ConversationId>,
) -> EventSelection {
    let conversation_id =
        requested_conversation_id.or_else(|| events.first().map(|event| event.conversation_id));
    let mut diagnostics = Vec::new();

    if events.is_empty() {
        diagnostics.push("no runtime events supplied".to_string());
    }

    if requested_conversation_id.is_none() && unique_conversation_count(events) > 1 {
        diagnostics.push("multiple conversations supplied; using first conversation".to_string());
    }

    let selected = match conversation_id {
        Some(conversation_id) => events
            .iter()
            .filter(|event| event.conversation_id == conversation_id)
            .cloned()
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    if conversation_id.is_some() && selected.is_empty() {
        diagnostics.push("no runtime events matched selected conversation".to_string());
    }

    EventSelection {
        conversation_id,
        events: selected,
        diagnostics,
    }
}

fn unique_conversation_count(events: &[RuntimeEvent]) -> usize {
    let mut conversations = Vec::new();
    for event in events {
        if !conversations.contains(&event.conversation_id) {
            conversations.push(event.conversation_id);
        }
    }
    conversations.len()
}

fn count_checkpoint_events(events: &[RuntimeEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event.payload, RuntimeEventPayload::CheckpointRecorded(_)))
        .count()
}

fn build_trace_evidence(
    conversation_id: Option<ConversationId>,
    events: &[RuntimeEvent],
) -> Result<Vec<TraceEntry>, ObservabilityEvidenceError> {
    let Some(conversation_id) = conversation_id else {
        return Ok(Vec::new());
    };
    let mut store = InMemoryEventStore::default();
    for event in events {
        store.append(event.clone()).map_err(trace_storage_error)?;
    }

    build_trace(
        &store,
        TraceQuery {
            conversation_id,
            include_tools: true,
            include_policy: true,
            include_checkpoints: true,
            include_final: true,
        },
    )
    .map_err(trace_error)
}

fn build_replay_summary(events: &[RuntimeEvent]) -> ReplayEvidenceSummary {
    let mut summary = ReplayEvidenceSummary {
        replayable: !events.is_empty(),
        event_count: events.len(),
        ..ReplayEvidenceSummary::default()
    };

    for event in events {
        match &event.payload {
            RuntimeEventPayload::CheckpointRecorded(_) => summary.checkpoint_count += 1,
            RuntimeEventPayload::ReplayStarted(_) => summary.replay_started_count += 1,
            RuntimeEventPayload::ReplayCompleted(_) => summary.replay_completed_count += 1,
            _ => {}
        }
    }

    if events.is_empty() {
        summary
            .diagnostics
            .push("no runtime events available for replay evidence".to_string());
    }
    if summary.checkpoint_count == 0 && !events.is_empty() {
        summary
            .diagnostics
            .push("no checkpoints recorded for replay evidence".to_string());
    }
    if summary.replay_started_count != summary.replay_completed_count {
        summary.diagnostics.push(format!(
            "replay start/completion mismatch: {} started, {} completed",
            summary.replay_started_count, summary.replay_completed_count
        ));
    }

    summary
}

fn trace_storage_error(error: agent_storage::StorageError) -> ObservabilityEvidenceError {
    ObservabilityEvidenceError::Trace(error.to_string())
}

fn trace_error(error: TraceError) -> ObservabilityEvidenceError {
    ObservabilityEvidenceError::Trace(error.to_string())
}

fn audit_error(error: AuditExportError) -> ObservabilityEvidenceError {
    ObservabilityEvidenceError::Audit(error.to_string())
}

fn telemetry_error(error: TelemetryExportError) -> ObservabilityEvidenceError {
    ObservabilityEvidenceError::Telemetry(error.to_string())
}
