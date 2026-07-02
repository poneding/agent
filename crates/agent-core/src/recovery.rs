use std::collections::{BTreeMap, BTreeSet};

use agent_storage::{
    CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore, ReplayCursor, StorageError,
};
use agent_types::{
    CheckpointId, ConversationId, EventSeq, RecoveredApprovalWait, RecoveredToolCall,
    RecoveredToolCallStatus, RecoveredToolEffect, RecoveredToolEffectDisposition,
    RecoveryCheckpointSummary, RecoveryDiagnostic, RecoveryDiagnosticSeverity, RecoveryReadiness,
    RecoveryReplaySummary, RecoverySnapshot, ResumeDryRunReport, RuntimeEventPayload,
    ToolEffectDisposition, TurnId,
};
use thiserror::Error;

use crate::ApprovalRecoveryView;

const MAX_RECOVERY_DIAGNOSTIC_CHARS: usize = 240;
const MAX_RECOVERY_SUMMARY_CHARS: usize = 240;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPlan {
    pub conversation_id: ConversationId,
    pub checkpoint_id: Option<CheckpointId>,
    pub replay_from: EventSeq,
    pub pending_call_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryOutcome {
    pub plan: RecoveryPlan,
    pub replayed_event_count: usize,
    pub resumed: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RecoveryError {
    #[error("storage failed: {0}")]
    Storage(String),
    #[error("checkpoint state is corrupt: {0}")]
    CorruptCheckpoint(String),
}

#[derive(Clone, Debug, Default)]
pub struct RecoveryEngine;

impl RecoveryEngine {
    pub fn plan<S>(
        &self,
        store: &S,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<RecoveryPlan, RecoveryError>
    where
        S: CheckpointStore + ?Sized,
    {
        let checkpoint = store
            .latest_checkpoint(conversation_id, turn_id)
            .map_err(storage_error)?;
        if let Some(checkpoint) = checkpoint {
            validate_checkpoint_state(&checkpoint.state_json)?;
            return Ok(RecoveryPlan {
                conversation_id,
                checkpoint_id: Some(checkpoint.checkpoint_id),
                replay_from: next_sequence(checkpoint.event_seq)?,
                pending_call_ids: Vec::new(),
            });
        }

        Ok(RecoveryPlan {
            conversation_id,
            checkpoint_id: None,
            replay_from: EventSeq::new(1).expect("one is a valid replay sequence"),
            pending_call_ids: Vec::new(),
        })
    }

    pub fn recover<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> Result<RecoveryOutcome, RecoveryError>
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let mut plan = self.plan(events, conversation_id, turn_id)?;
        let replayed = ReplayCursor::from_sequence(conversation_id, plan.replay_from)
            .collect(events)
            .map_err(storage_error)?;
        plan.pending_call_ids = pending_call_ids(&replayed, idempotency)?;

        Ok(RecoveryOutcome {
            replayed_event_count: replayed.len(),
            resumed: true,
            plan,
        })
    }

    pub fn snapshot<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> RecoverySnapshot
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let mut diagnostics = Vec::new();
        let (checkpoint, replay_from) =
            match recovery_checkpoint(events, conversation_id, turn_id, &mut diagnostics) {
                Some((checkpoint, replay_from)) => (Some(checkpoint), replay_from),
                None => (
                    None,
                    EventSeq::new(1).expect("one is a valid replay sequence"),
                ),
            };

        let replayed = match ReplayCursor::from_sequence(conversation_id, replay_from)
            .collect(events)
            .map_err(storage_error)
        {
            Ok(replayed) => replayed,
            Err(error) => {
                diagnostics.push(diagnostic(
                    RecoveryDiagnosticSeverity::Error,
                    "recovery.replay.storage_failed",
                    error.to_string(),
                    Some("event_store"),
                ));
                Vec::new()
            }
        };

        let (pending_tool_calls, idempotent_effects) =
            recovered_tools(&replayed, idempotency, &mut diagnostics);
        let idempotent_recovery = idempotent_recovery_summary(
            &pending_tool_calls,
            &idempotent_effects,
            diagnostics.clone(),
        );

        let pending_approvals = recovered_approvals(events, conversation_id, &mut diagnostics);
        let readiness = readiness(&diagnostics, &pending_approvals, &pending_tool_calls);

        RecoverySnapshot {
            conversation_id,
            turn_id,
            checkpoint,
            replay: RecoveryReplaySummary {
                replay_from,
                replayed_event_count: usize_to_u32(replayed.len()),
            },
            readiness,
            pending_tool_calls,
            idempotent_effects,
            idempotent_recovery,
            pending_approvals,
            diagnostics,
        }
    }

    pub fn resume_dry_run<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> ResumeDryRunReport
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let snapshot = self.snapshot(events, idempotency, conversation_id, turn_id);
        ResumeDryRunReport {
            will_replay_event_count: snapshot.replay.replayed_event_count,
            will_resume_pending_tool_count: usize_to_u32(snapshot.pending_tool_calls.len()),
            will_reuse_idempotent_effect_count: usize_to_u32(snapshot.idempotent_effects.len()),
            blocked: !matches!(
                snapshot.readiness,
                RecoveryReadiness::Ready
                    | RecoveryReadiness::NeedsApproval
                    | RecoveryReadiness::PendingToolCalls
            ),
            snapshot,
        }
    }
}

fn recovery_checkpoint<S>(
    store: &S,
    conversation_id: ConversationId,
    turn_id: Option<TurnId>,
    diagnostics: &mut Vec<RecoveryDiagnostic>,
) -> Option<(RecoveryCheckpointSummary, EventSeq)>
where
    S: CheckpointStore + ?Sized,
{
    let checkpoint = match store.latest_checkpoint(conversation_id, turn_id) {
        Ok(checkpoint) => checkpoint,
        Err(error) => {
            diagnostics.push(diagnostic(
                RecoveryDiagnosticSeverity::Error,
                "recovery.checkpoint.storage_failed",
                error.to_string(),
                Some("checkpoint_store"),
            ));
            return None;
        }
    };

    let checkpoint = checkpoint?;

    if let Err(error) = validate_checkpoint_state(&checkpoint.state_json) {
        diagnostics.push(diagnostic(
            RecoveryDiagnosticSeverity::Error,
            "recovery.checkpoint.corrupt",
            error.to_string(),
            Some("checkpoint_store"),
        ));
        return Some((
            RecoveryCheckpointSummary {
                checkpoint_id: checkpoint.checkpoint_id,
                event_seq: checkpoint.event_seq,
                label: bound_summary(checkpoint.label),
                created_at: checkpoint.created_at,
            },
            EventSeq::new(1).expect("one is a valid replay sequence"),
        ));
    }

    let replay_from = match next_sequence(checkpoint.event_seq) {
        Ok(seq) => seq,
        Err(error) => {
            diagnostics.push(diagnostic(
                RecoveryDiagnosticSeverity::Error,
                "recovery.checkpoint.invalid_sequence",
                error.to_string(),
                Some("checkpoint_store"),
            ));
            EventSeq::new(1).expect("one is a valid replay sequence")
        }
    };

    Some((
        RecoveryCheckpointSummary {
            checkpoint_id: checkpoint.checkpoint_id,
            event_seq: checkpoint.event_seq,
            label: bound_summary(checkpoint.label),
            created_at: checkpoint.created_at,
        },
        replay_from,
    ))
}

fn idempotent_recovery_summary(
    pending_tool_calls: &[RecoveredToolCall],
    idempotent_effects: &[RecoveredToolEffect],
    diagnostics: Vec<RecoveryDiagnostic>,
) -> agent_types::IdempotentRecoverySummary {
    agent_types::IdempotentRecoverySummary {
        reusable_effect_count: usize_to_u32(idempotent_effects.len()),
        pending_tool_count: usize_to_u32(pending_tool_calls.len()),
        conflict_count: usize_to_u32(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code.contains("idempotency.conflict"))
                .count(),
        ),
        effects: idempotent_effects
            .iter()
            .map(|effect| RecoveredToolEffectDisposition {
                call_id: effect.call_id.clone(),
                tool_name: effect.tool_name.clone(),
                idempotency_key: effect.idempotency_key.clone(),
                input_hash: effect.input_hash.clone(),
                disposition: ToolEffectDisposition::Reusable,
                completed_at: effect.completed_at,
            })
            .collect(),
        diagnostics: diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.code.contains("idempotency"))
            .collect(),
    }
}

fn recovered_tools<I>(
    replayed: &[agent_storage::ReplayEvent],
    idempotency: &I,
    diagnostics: &mut Vec<RecoveryDiagnostic>,
) -> (Vec<RecoveredToolCall>, Vec<RecoveredToolEffect>)
where
    I: IdempotencyStore + ?Sized,
{
    let mut proposed = BTreeMap::new();
    let mut terminal = BTreeSet::new();

    for replay in replayed {
        match &replay.event.payload {
            RuntimeEventPayload::ToolCallProposed(call) => {
                proposed.insert(call.call_id.clone(), call.tool_name.clone());
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                terminal.insert(completed.call_id.clone());
            }
            RuntimeEventPayload::ToolExecutionFailed(failed) => {
                terminal.insert(failed.call_id.clone());
            }
            RuntimeEventPayload::ToolObservationRecorded(observed) => {
                terminal.insert(observed.call_id.clone());
            }
            _ => {}
        }
    }

    let mut pending = Vec::new();
    let mut effects = Vec::new();
    for (call_id, tool_name) in proposed {
        if terminal.contains(&call_id) {
            continue;
        }
        let key = match IdempotencyKey::new(call_id.clone()) {
            Ok(key) => key,
            Err(error) => {
                diagnostics.push(diagnostic(
                    RecoveryDiagnosticSeverity::Error,
                    "recovery.idempotency.invalid_key",
                    error.to_string(),
                    Some("idempotency_store"),
                ));
                pending.push(RecoveredToolCall {
                    call_id,
                    tool_name: Some(tool_name),
                    status: RecoveredToolCallStatus::Pending,
                });
                continue;
            }
        };
        match idempotency.get_effect(&key) {
            Ok(Some(record)) => effects.push(RecoveredToolEffect {
                idempotency_key: record.key.to_string(),
                call_id: record.call_id,
                tool_name: record.tool_name,
                input_hash: bound_summary(record.input_hash),
                completed_at: record.completed_at,
            }),
            Ok(None) => pending.push(RecoveredToolCall {
                call_id,
                tool_name: Some(tool_name),
                status: RecoveredToolCallStatus::Pending,
            }),
            Err(error) => {
                diagnostics.push(diagnostic(
                    RecoveryDiagnosticSeverity::Error,
                    "recovery.idempotency.storage_failed",
                    error.to_string(),
                    Some("idempotency_store"),
                ));
                pending.push(RecoveredToolCall {
                    call_id,
                    tool_name: Some(tool_name),
                    status: RecoveredToolCallStatus::Pending,
                });
            }
        }
    }
    (pending, effects)
}

fn recovered_approvals<E>(
    events: &E,
    conversation_id: ConversationId,
    diagnostics: &mut Vec<RecoveryDiagnostic>,
) -> Vec<RecoveredApprovalWait>
where
    E: EventStore,
{
    let all_events = match ReplayCursor::new(conversation_id).collect(events) {
        Ok(events) => events,
        Err(error) => {
            diagnostics.push(diagnostic(
                RecoveryDiagnosticSeverity::Error,
                "recovery.approval.storage_failed",
                error.to_string(),
                Some("event_store"),
            ));
            return Vec::new();
        }
    };
    ApprovalRecoveryView::pending_from_events(all_events.iter().map(|event| &event.event))
        .into_iter()
        .map(|approval| RecoveredApprovalWait {
            approval_id: approval.request_id.to_string(),
            checkpoint_id: approval.checkpoint_id,
            action: bound_summary(approval.action),
            resource_kind: bound_summary(approval.resource_kind),
            resource: bound_summary(approval.resource),
            summary: bound_summary(approval.reason),
        })
        .collect()
}

fn readiness(
    diagnostics: &[RecoveryDiagnostic],
    pending_approvals: &[RecoveredApprovalWait],
    pending_tool_calls: &[RecoveredToolCall],
) -> RecoveryReadiness {
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "recovery.checkpoint.corrupt")
    {
        return RecoveryReadiness::CorruptCheckpoint;
    }
    if diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == RecoveryDiagnosticSeverity::Error
            && diagnostic.code.contains("storage_failed")
    }) {
        return RecoveryReadiness::StorageError;
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == RecoveryDiagnosticSeverity::Error)
    {
        return RecoveryReadiness::Blocked;
    }
    if !pending_approvals.is_empty() {
        return RecoveryReadiness::NeedsApproval;
    }
    if !pending_tool_calls.is_empty() {
        return RecoveryReadiness::PendingToolCalls;
    }
    RecoveryReadiness::Ready
}

fn diagnostic(
    severity: RecoveryDiagnosticSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
    source: Option<&str>,
) -> RecoveryDiagnostic {
    let (message, truncated) = bound_text(message, MAX_RECOVERY_DIAGNOSTIC_CHARS);
    RecoveryDiagnostic {
        severity,
        code: code.into(),
        message,
        source: source.map(ToOwned::to_owned),
        truncated,
    }
}

fn bound_summary(value: impl Into<String>) -> String {
    bound_text(value, MAX_RECOVERY_SUMMARY_CHARS).0
}

fn bound_text(value: impl Into<String>, max_chars: usize) -> (String, bool) {
    let value = value.into();
    let redacted = redact_secret_like(&value);
    if redacted.chars().count() <= max_chars {
        return (redacted, false);
    }
    let mut output = redacted.chars().take(max_chars).collect::<String>();
    output.push_str("...");
    (output, true)
}

fn redact_secret_like(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if lower.contains("secret")
                || lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("password")
            {
                "[redacted]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn pending_call_ids<I>(
    replayed: &[agent_storage::ReplayEvent],
    idempotency: &I,
) -> Result<Vec<String>, RecoveryError>
where
    I: IdempotencyStore + ?Sized,
{
    let mut proposed = BTreeSet::new();
    let mut terminal = BTreeSet::new();

    for replay in replayed {
        match &replay.event.payload {
            RuntimeEventPayload::ToolCallProposed(call) => {
                proposed.insert(call.call_id.clone());
            }
            RuntimeEventPayload::ToolExecutionCompleted(completed) => {
                terminal.insert(completed.call_id.clone());
            }
            RuntimeEventPayload::ToolExecutionFailed(failed) => {
                terminal.insert(failed.call_id.clone());
            }
            RuntimeEventPayload::ToolObservationRecorded(observed) => {
                terminal.insert(observed.call_id.clone());
            }
            _ => {}
        }
    }

    let mut pending = Vec::new();
    for call_id in proposed {
        if terminal.contains(&call_id) {
            continue;
        }
        let key = IdempotencyKey::new(call_id.clone()).map_err(storage_error)?;
        if idempotency
            .get_effect(&key)
            .map_err(storage_error)?
            .is_some()
        {
            continue;
        }
        pending.push(call_id);
    }
    Ok(pending)
}

fn validate_checkpoint_state(value: &serde_json::Value) -> Result<(), RecoveryError> {
    if !value.is_object() {
        return Err(RecoveryError::CorruptCheckpoint(
            "checkpoint state must be a JSON object".to_string(),
        ));
    }
    if value
        .get("corrupt")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Err(RecoveryError::CorruptCheckpoint(
            "checkpoint state is corrupt".to_string(),
        ));
    }
    Ok(())
}

fn next_sequence(seq: EventSeq) -> Result<EventSeq, RecoveryError> {
    EventSeq::new(seq.get() + 1)
        .map_err(|error| RecoveryError::CorruptCheckpoint(error.to_string()))
}

fn storage_error(error: StorageError) -> RecoveryError {
    RecoveryError::Storage(error.to_string())
}
