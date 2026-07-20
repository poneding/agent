use agent_storage::{
    CheckpointRecord, CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore,
    InMemoryEventStore, ToolEffectRecord,
};
use agent_types::{
    ActorRef, CheckpointId, CommandRetryPlan, CommandRetryReadiness, CommandRetryRequest,
    ConversationId, CorrelationId, EventSeq, RecoveryReadiness, RecoverySnapshot, RuntimeEvent,
    RuntimeEventPayload, SchemaVersion, Timestamp, ToolCallProposed, TurnId, UserInputReceived,
    WorkspaceId,
};
use agent_worktree::{ChangeKind, FileChange, detect_conflicts};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    BoundedCommandOutput, CommandCancellationState, CommandExecutionStatus,
    CommandFailureClassification, CommandIntentSummary, CommandObservationSummary,
    CommandRetryPlanner, CommandTestLoopReport, ConflictRecoveryMapper, RecoveryEngine,
};

const SECRET_VALUE: &str = "token-secret-api_key-password";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryCrashSmokeReport {
    pub snapshot: RecoverySnapshot,
    pub dry_run: agent_types::ResumeDryRunReport,
    pub retry_plan: CommandRetryPlan,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RecoveryCrashSmokeError {
    #[error("fixture setup failed: {0}")]
    Fixture(String),
    #[error("fixture verification failed: {0}")]
    Verification(String),
    #[error("fixture serialization failed: {0}")]
    Serialization(String),
}

pub fn run_recovery_crash_smoke_fixture()
-> Result<RecoveryCrashSmokeReport, RecoveryCrashSmokeError> {
    let workspace_id = WorkspaceId::new();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let checkpoint_id = CheckpointId::new();

    let mut pre_crash_store = InMemoryEventStore::default();
    pre_crash_store
        .append(user_event(
            workspace_id,
            conversation_id,
            turn_id,
            "start recovery smoke",
        ))
        .map_err(fixture_error)?;
    pre_crash_store
        .put_checkpoint(checkpoint(checkpoint_id, conversation_id, turn_id, 1))
        .map_err(fixture_error)?;
    pre_crash_store
        .append(tool_call_event(
            workspace_id,
            conversation_id,
            turn_id,
            "call-pending",
            "workspace.edit_file",
        ))
        .map_err(fixture_error)?;
    pre_crash_store
        .append(tool_call_event(
            workspace_id,
            conversation_id,
            turn_id,
            "call-reusable",
            "workspace.write_file",
        ))
        .map_err(fixture_error)?;
    pre_crash_store
        .record_effect(ToolEffectRecord {
            key: IdempotencyKey::new("call-reusable").map_err(fixture_error)?,
            call_id: "call-reusable".to_string(),
            tool_name: "workspace.write_file".to_string(),
            input_hash: format!("fnv1a64:{SECRET_VALUE}"),
            output_json: json!({ "raw": SECRET_VALUE }),
            completed_at: Timestamp::now_utc(),
        })
        .map_err(fixture_error)?;

    let persisted_store = pre_crash_store;
    let recovery_engine_after_restart = RecoveryEngine;
    let conflicts = conflict_summaries()?;

    let snapshot = recovery_engine_after_restart.snapshot_with_conflicts(
        &persisted_store,
        &persisted_store,
        conversation_id,
        Some(turn_id),
        conflicts.clone(),
    );
    let dry_run = recovery_engine_after_restart.resume_dry_run_with_conflicts(
        &persisted_store,
        &persisted_store,
        conversation_id,
        Some(turn_id),
        conflicts,
    );
    let retry_plan = CommandRetryPlanner.plan_from_report(
        CommandRetryRequest {
            command_run_id: format!("cmd-recovery-{SECRET_VALUE}"),
            reason: format!("retry after crash {SECRET_VALUE}"),
            dry_run: true,
        },
        &failed_command_report(workspace_id, conversation_id),
    );

    let report = RecoveryCrashSmokeReport {
        snapshot,
        dry_run,
        retry_plan,
    };
    verify_report(&report, checkpoint_id)?;
    Ok(report)
}

fn verify_report(
    report: &RecoveryCrashSmokeReport,
    checkpoint_id: CheckpointId,
) -> Result<(), RecoveryCrashSmokeError> {
    if report
        .snapshot
        .checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.checkpoint_id)
        != Some(checkpoint_id)
    {
        return Err(RecoveryCrashSmokeError::Verification(
            "checkpoint metadata was not recovered".to_string(),
        ));
    }
    if report.snapshot.replay.replayed_event_count == 0 {
        return Err(RecoveryCrashSmokeError::Verification(
            "replay tail count was zero".to_string(),
        ));
    }
    if report.snapshot.pending_tool_calls.len() != 1 {
        return Err(RecoveryCrashSmokeError::Verification(
            "pending tool count was not preserved".to_string(),
        ));
    }
    if report.snapshot.idempotent_effects.len() != 1
        || report.snapshot.idempotent_recovery.reusable_effect_count != 1
    {
        return Err(RecoveryCrashSmokeError::Verification(
            "idempotent effect reuse evidence was missing".to_string(),
        ));
    }
    if report.snapshot.readiness != RecoveryReadiness::HasConflict || !report.dry_run.blocked {
        return Err(RecoveryCrashSmokeError::Verification(
            "conflicts did not block resume dry-run".to_string(),
        ));
    }
    if report.retry_plan.readiness != CommandRetryReadiness::Retryable || !report.retry_plan.dry_run
    {
        return Err(RecoveryCrashSmokeError::Verification(
            "retry plan was not retryable dry-run evidence".to_string(),
        ));
    }

    let serialized = serde_json::to_string(report)
        .map_err(|error| RecoveryCrashSmokeError::Serialization(error.to_string()))?;
    for forbidden in ["token-secret", "api_key", "password", SECRET_VALUE] {
        if serialized.contains(forbidden) {
            return Err(RecoveryCrashSmokeError::Verification(format!(
                "serialized evidence leaked {forbidden}"
            )));
        }
    }
    Ok(())
}

fn runtime_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    payload: RuntimeEventPayload,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id,
        conversation_id,
        turn_id: Some(turn_id),
        task_id: None,
        agent_id: None,
        team_id: None,
        actor: ActorRef::system(),
        correlation_id: CorrelationId::new(),
        causation_id: None,
        payload,
    }
}

fn user_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    message: &str,
) -> RuntimeEvent {
    runtime_event(
        workspace_id,
        conversation_id,
        turn_id,
        RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    )
}

fn tool_call_event(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    call_id: &str,
    tool_name: &str,
) -> RuntimeEvent {
    runtime_event(
        workspace_id,
        conversation_id,
        turn_id,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            input_json: json!({
                "path": "src/token-secret.rs",
                "value": SECRET_VALUE,
            }),
        }),
    )
}

fn checkpoint(
    checkpoint_id: CheckpointId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    event_seq: u64,
) -> CheckpointRecord {
    CheckpointRecord {
        checkpoint_id,
        conversation_id,
        turn_id: Some(turn_id),
        label: format!("before crash {SECRET_VALUE}"),
        event_seq: EventSeq::new(event_seq).expect("event seq"),
        state_json: json!({
            "phase": "before_crash",
            "raw": SECRET_VALUE,
        }),
        schema_version: SchemaVersion::current(),
        created_at: Timestamp::now_utc(),
    }
}

fn failed_command_report(
    workspace_id: WorkspaceId,
    conversation_id: ConversationId,
) -> CommandTestLoopReport {
    CommandTestLoopReport {
        workspace_id,
        conversation_id,
        command_run_id: "recovery-smoke-command-failed".to_string(),
        command: CommandIntentSummary {
            display: format!("cargo test recovery -- {SECRET_VALUE}"),
            program: Some("cargo".to_string()),
            args: vec![
                "test".to_string(),
                "recovery".to_string(),
                "--".to_string(),
                "[redacted]".to_string(),
            ],
            timeout_ms: 30_000,
            cwd: ".".to_string(),
            classify_as_test: true,
        },
        status: CommandExecutionStatus::Failed,
        classification: CommandFailureClassification::TestFailure,
        exit_code: Some(101),
        stdout: BoundedCommandOutput {
            text: "running recovery smoke".to_string(),
            byte_count: 22,
            truncated: false,
            redacted: false,
        },
        stderr: BoundedCommandOutput {
            text: "[redacted]".to_string(),
            byte_count: 10,
            truncated: false,
            redacted: true,
        },
        timed_out: false,
        cancellation: CommandCancellationState {
            cancellable: true,
            reason: "command can be retried after recovery".to_string(),
        },
        observation: CommandObservationSummary {
            message: "test failed with recoverable assertion".to_string(),
            truncated: false,
            redacted: false,
        },
        events: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn conflict_summaries() -> Result<Vec<agent_types::RecoveryConflictSummary>, RecoveryCrashSmokeError>
{
    let lead = vec![
        FileChange::new(
            "src/token-secret.rs",
            ChangeKind::Modified,
            Some("lead-token-secret-hash".to_string()),
        )
        .map_err(fixture_error)?,
    ];
    let incoming = vec![
        FileChange::new(
            "src/token-secret.rs",
            ChangeKind::Modified,
            Some("incoming-api_key-hash".to_string()),
        )
        .map_err(fixture_error)?,
    ];
    let report = detect_conflicts(&lead, &incoming).map_err(fixture_error)?;
    Ok(ConflictRecoveryMapper.map_report(&report))
}

fn fixture_error(error: impl std::fmt::Display) -> RecoveryCrashSmokeError {
    RecoveryCrashSmokeError::Fixture(error.to_string())
}
