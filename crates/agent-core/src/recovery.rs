use std::collections::{BTreeMap, BTreeSet};

use agent_storage::{
    CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore, ReplayCursor, StorageError,
};
use agent_types::{
    CheckpointId, ConversationId, EventSeq, RecoveredApprovalWait, RecoveredToolCall,
    RecoveredToolCallStatus, RecoveredToolEffect, RecoveredToolEffectDisposition,
    RecoveryCheckpointSummary, RecoveryContinuation, RecoveryDiagnostic,
    RecoveryDiagnosticSeverity, RecoveryReadiness, RecoveryReplaySummary, RecoverySnapshot,
    ResumeDryRunReport, ResumeReport, RuntimeEventPayload, ToolEffectDisposition, TurnId,
};
use thiserror::Error;

use crate::ApprovalRecoveryView;
use crate::ConflictRecoveryMapper;

const MAX_RECOVERY_DIAGNOSTIC_CHARS: usize = 240;
const MAX_RECOVERY_SUMMARY_CHARS: usize = 240;
const MAX_RECOVERY_CONTINUATION_SUMMARY_CHARS: usize = 480;
const MAX_RECOVERY_CONTINUATION_PROMPT_CHARS: usize = 1600;
const MAX_RECOVERY_CONTINUATION_ITEM_COUNT: usize = 8;

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
            conflicts: Vec::new(),
            pending_approvals,
            diagnostics,
        }
    }

    pub fn snapshot_with_conflicts<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
        conflicts: Vec<agent_types::RecoveryConflictSummary>,
    ) -> RecoverySnapshot
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let snapshot = self.snapshot(events, idempotency, conversation_id, turn_id);
        ConflictRecoveryMapper.apply_to_snapshot(snapshot, conflicts)
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

    pub fn resume<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
    ) -> ResumeReport
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let snapshot = self.snapshot(events, idempotency, conversation_id, turn_id);
        let blocked = !matches!(
            snapshot.readiness,
            RecoveryReadiness::Ready
                | RecoveryReadiness::NeedsApproval
                | RecoveryReadiness::PendingToolCalls
        );
        let resumed = !blocked
            && self
                .recover(events, idempotency, conversation_id, turn_id)
                .map(|outcome| outcome.resumed)
                .unwrap_or(false);
        let continuation = resumed
            .then(|| recovery_continuation(events, &snapshot))
            .flatten();

        ResumeReport {
            replayed_event_count: snapshot.replay.replayed_event_count,
            resumed_pending_tool_count: usize_to_u32(snapshot.pending_tool_calls.len()),
            reused_idempotent_effect_count: usize_to_u32(snapshot.idempotent_effects.len()),
            resumed,
            blocked,
            continuation,
            snapshot,
        }
    }

    pub fn resume_dry_run_with_conflicts<E, I>(
        &self,
        events: &E,
        idempotency: &I,
        conversation_id: ConversationId,
        turn_id: Option<TurnId>,
        conflicts: Vec<agent_types::RecoveryConflictSummary>,
    ) -> ResumeDryRunReport
    where
        E: EventStore + CheckpointStore,
        I: IdempotencyStore + ?Sized,
    {
        let snapshot =
            self.snapshot_with_conflicts(events, idempotency, conversation_id, turn_id, conflicts);
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

fn recovery_continuation<E>(events: &E, snapshot: &RecoverySnapshot) -> Option<RecoveryContinuation>
where
    E: EventStore,
{
    let replayed =
        ReplayCursor::from_sequence(snapshot.conversation_id, snapshot.replay.replay_from)
            .collect(events)
            .ok()?;

    let mut last_user_message = None;
    let mut last_final_response = None;
    let mut last_turn_summary = None;
    let mut replay_tool_names = BTreeSet::new();

    for replay in &replayed {
        match &replay.event.payload {
            RuntimeEventPayload::UserInputReceived(input) => {
                last_user_message = Some(bound_summary(input.message.clone()));
            }
            RuntimeEventPayload::FinalResponseProduced(response) => {
                last_final_response = Some(bound_summary(response.message.clone()));
            }
            RuntimeEventPayload::TurnCompleted(completed) => {
                last_turn_summary = Some(bound_summary(completed.summary.clone()));
            }
            RuntimeEventPayload::ToolCallProposed(call) => {
                replay_tool_names.insert(bound_summary(call.tool_name.clone()));
            }
            _ => {}
        }
    }

    let checkpoint = snapshot
        .checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.checkpoint_id.to_string())
        .unwrap_or_else(|| "none".to_string());
    let pending_tool_count = usize_to_u32(snapshot.pending_tool_calls.len());
    let reused_effect_count = usize_to_u32(snapshot.idempotent_effects.len());
    let pending_approval_count = usize_to_u32(snapshot.pending_approvals.len());
    let can_start_turn = matches!(snapshot.readiness, RecoveryReadiness::Ready);

    let summary_raw = format!(
        "readiness={} checkpoint={} replayed={} pending_tools={} reusable_effects={} pending_approvals={}",
        readiness_label(snapshot.readiness),
        checkpoint,
        snapshot.replay.replayed_event_count,
        pending_tool_count,
        reused_effect_count,
        pending_approval_count
    );
    let (summary, summary_truncated) =
        bound_text(summary_raw, MAX_RECOVERY_CONTINUATION_SUMMARY_CHARS);

    let mut lines = vec![
        "Recover the interrupted agent turn using this runtime replay state.".to_string(),
        format!("conversation_id: {}", snapshot.conversation_id),
        format!("turn_id: {}", optional_turn_id(snapshot.turn_id)),
        format!("checkpoint_id: {checkpoint}"),
        format!(
            "readiness: {} replayed_events: {}",
            readiness_label(snapshot.readiness),
            snapshot.replay.replayed_event_count
        ),
        format!(
            "pending_tools: {} reusable_effects: {} pending_approvals: {}",
            pending_tool_count, reused_effect_count, pending_approval_count
        ),
    ];

    if let Some(message) = last_user_message {
        lines.push(format!("last_user_message: {message}"));
    }
    if let Some(response) = last_final_response {
        lines.push(format!("last_final_response: {response}"));
    }
    if let Some(turn_summary) = last_turn_summary {
        lines.push(format!("last_turn_summary: {turn_summary}"));
    }

    push_limited_list(
        &mut lines,
        "replayed_tools",
        replay_tool_names,
        MAX_RECOVERY_CONTINUATION_ITEM_COUNT,
    );
    push_limited_list(
        &mut lines,
        "pending_tool_calls",
        snapshot.pending_tool_calls.iter().map(|call| {
            let tool = call
                .tool_name
                .as_deref()
                .map(bound_summary)
                .unwrap_or_else(|| "unknown".to_string());
            format!("{}:{tool}", bound_summary(call.call_id.clone()))
        }),
        MAX_RECOVERY_CONTINUATION_ITEM_COUNT,
    );
    push_limited_list(
        &mut lines,
        "reusable_effects",
        snapshot.idempotent_effects.iter().map(|effect| {
            format!(
                "{}:{}",
                bound_summary(effect.call_id.clone()),
                bound_summary(effect.tool_name.clone())
            )
        }),
        MAX_RECOVERY_CONTINUATION_ITEM_COUNT,
    );
    push_limited_list(
        &mut lines,
        "pending_approvals",
        snapshot.pending_approvals.iter().map(|approval| {
            format!(
                "{}:{}:{}",
                bound_summary(approval.approval_id.clone()),
                bound_summary(approval.action.clone()),
                bound_summary(approval.resource.clone())
            )
        }),
        MAX_RECOVERY_CONTINUATION_ITEM_COUNT,
    );

    lines.push(recovery_next_action(snapshot.readiness).to_string());
    lines.push(
        "Do not repeat reusable tool effects; continue from replayed state and preserve policy checks."
            .to_string(),
    );

    let (next_prompt, prompt_truncated) =
        bound_text(lines.join("\n"), MAX_RECOVERY_CONTINUATION_PROMPT_CHARS);

    Some(RecoveryContinuation {
        summary,
        next_prompt,
        can_start_turn,
        replayed_event_count: snapshot.replay.replayed_event_count,
        pending_tool_count,
        reused_effect_count,
        pending_approval_count,
        summary_truncated,
        prompt_truncated,
    })
}

fn optional_turn_id(turn_id: Option<TurnId>) -> String {
    turn_id
        .map(|turn_id| turn_id.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn push_limited_list<I>(lines: &mut Vec<String>, label: &str, values: I, max_items: usize)
where
    I: IntoIterator<Item = String>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }
    values.sort();
    let omitted = values.len().saturating_sub(max_items);
    let mut rendered = values.into_iter().take(max_items).collect::<Vec<_>>();
    if omitted > 0 {
        rendered.push(format!("... {omitted} more"));
    }
    lines.push(format!("{label}: {}", rendered.join(", ")));
}

fn recovery_next_action(readiness: RecoveryReadiness) -> &'static str {
    match readiness {
        RecoveryReadiness::Ready => "Next action: continue the interrupted turn.",
        RecoveryReadiness::NeedsApproval => {
            "Next action: restore the pending approval workflow before continuing the turn."
        }
        RecoveryReadiness::PendingToolCalls => {
            "Next action: resume pending tool calls through the runtime tool policy path."
        }
        RecoveryReadiness::HasConflict => "Next action: inspect conflicts before continuing.",
        RecoveryReadiness::CorruptCheckpoint => "Next action: repair or choose another checkpoint.",
        RecoveryReadiness::StorageError => "Next action: repair storage before continuing.",
        RecoveryReadiness::Blocked => "Next action: resolve recovery blockers before continuing.",
    }
}

fn readiness_label(readiness: RecoveryReadiness) -> &'static str {
    match readiness {
        RecoveryReadiness::Ready => "ready",
        RecoveryReadiness::NeedsApproval => "needs_approval",
        RecoveryReadiness::PendingToolCalls => "pending_tool_calls",
        RecoveryReadiness::HasConflict => "has_conflict",
        RecoveryReadiness::CorruptCheckpoint => "corrupt_checkpoint",
        RecoveryReadiness::StorageError => "storage_error",
        RecoveryReadiness::Blocked => "blocked",
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
