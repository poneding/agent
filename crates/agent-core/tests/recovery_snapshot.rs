use agent_core::RecoveryEngine;
use agent_storage::{
    CheckpointRecord, CheckpointStore, EventStore, IdempotencyKey, IdempotencyStore,
    InMemoryEventStore, ToolEffectRecord,
};
use agent_types::{
    ActorRef, ApprovalPreviewRecorded, ApprovalRequested, CheckpointId, ConversationId,
    CorrelationId, EventSeq, RecoveredToolCallStatus, RecoveryDiagnosticSeverity,
    RecoveryReadiness, RuntimeEvent, RuntimeEventPayload, SchemaVersion, Timestamp,
    ToolCallProposed, ToolExecutionCompleted, TurnId, UserInputReceived, WorkspaceId,
};
use serde_json::json;

fn runtime_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    payload: RuntimeEventPayload,
) -> RuntimeEvent {
    RuntimeEvent {
        schema_version: SchemaVersion::current(),
        id: agent_types::EventId::new(),
        seq: None,
        occurred_at: Timestamp::now_utc(),
        workspace_id: WorkspaceId::new(),
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

fn user_event(conversation_id: ConversationId, turn_id: TurnId, message: &str) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::UserInputReceived(UserInputReceived {
            message: message.to_string(),
        }),
    )
}

fn tool_call_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    call_id: &str,
    tool_name: &str,
) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::ToolCallProposed(ToolCallProposed {
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            input_json: json!({
                "path": "README.md",
                "secret_token": "must-not-appear"
            }),
        }),
    )
}

fn tool_completed_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    call_id: &str,
) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::ToolExecutionCompleted(ToolExecutionCompleted {
            call_id: call_id.to_string(),
            output_json: json!({ "raw": "must-not-appear" }),
        }),
    )
}

fn approval_requested_event(
    conversation_id: ConversationId,
    turn_id: TurnId,
    checkpoint_id: Option<CheckpointId>,
) -> RuntimeEvent {
    runtime_event(
        conversation_id,
        turn_id,
        RuntimeEventPayload::ApprovalRequested(ApprovalRequested {
            request_id: agent_types::ApprovalRequestId::new(),
            status: "pending".to_string(),
            action: "write".to_string(),
            resource_kind: "tool".to_string(),
            resource: "workspace.edit_file".to_string(),
            reason: "needs user approval".to_string(),
            preview: ApprovalPreviewRecorded {
                kind: "diff".to_string(),
                resource: "README.md".to_string(),
                summary: "edit README".to_string(),
                content_hash: Some("sha256:preview".to_string()),
                truncated: false,
                redacted: true,
            },
            checkpoint_id,
            max_duration_seconds: Some(60),
            expires_at: None,
        }),
    )
}

fn checkpoint(
    checkpoint_id: CheckpointId,
    conversation_id: ConversationId,
    turn_id: TurnId,
    event_seq: u64,
    state_json: serde_json::Value,
) -> CheckpointRecord {
    CheckpointRecord {
        checkpoint_id,
        conversation_id,
        turn_id: Some(turn_id),
        label: format!("checkpoint-{event_seq}"),
        event_seq: EventSeq::new(event_seq).expect("event seq"),
        state_json,
        schema_version: SchemaVersion::current(),
        created_at: Timestamp::now_utc(),
    }
}

fn record_effect(store: &mut InMemoryEventStore, call_id: &str, tool_name: &str) {
    store
        .record_effect(ToolEffectRecord {
            key: IdempotencyKey::new(call_id).expect("idempotency key"),
            call_id: call_id.to_string(),
            tool_name: tool_name.to_string(),
            input_hash: "fnv1a64:abc123".to_string(),
            output_json: json!({ "raw_secret": "must-not-appear" }),
            completed_at: Timestamp::now_utc(),
        })
        .expect("record effect");
}

#[test]
fn recovery_snapshot_without_checkpoint_starts_from_sequence_one() {
    let store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    let snapshot = RecoveryEngine.snapshot(&store, &store, conversation_id, Some(turn_id));

    assert_eq!(snapshot.conversation_id, conversation_id);
    assert_eq!(snapshot.turn_id, Some(turn_id));
    assert_eq!(snapshot.checkpoint, None);
    assert_eq!(snapshot.replay.replay_from.get(), 1);
    assert_eq!(snapshot.replay.replayed_event_count, 0);
    assert_eq!(snapshot.readiness, RecoveryReadiness::Ready);
    assert!(snapshot.diagnostics.is_empty());
}

#[test]
fn recovery_snapshot_includes_checkpoint_metadata_and_tail_count() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let checkpoint_id = CheckpointId::new();

    store
        .append(user_event(conversation_id, turn_id, "before checkpoint"))
        .expect("append first");
    store
        .append(user_event(conversation_id, turn_id, "after checkpoint"))
        .expect("append second");
    store
        .put_checkpoint(checkpoint(
            checkpoint_id,
            conversation_id,
            turn_id,
            1,
            json!({ "state": "observed" }),
        ))
        .expect("checkpoint");

    let snapshot = RecoveryEngine.snapshot(&store, &store, conversation_id, Some(turn_id));

    let checkpoint = snapshot.checkpoint.expect("checkpoint summary");
    assert_eq!(checkpoint.checkpoint_id, checkpoint_id);
    assert_eq!(checkpoint.event_seq.get(), 1);
    assert_eq!(snapshot.replay.replay_from.get(), 2);
    assert_eq!(snapshot.replay.replayed_event_count, 1);
    assert_eq!(snapshot.readiness, RecoveryReadiness::Ready);
}

#[test]
fn recovery_snapshot_reports_corrupt_checkpoint_as_blocking_diagnostic() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "corrupt": true }),
        ))
        .expect("checkpoint");

    let dry_run = RecoveryEngine.resume_dry_run(&store, &store, conversation_id, Some(turn_id));

    assert_eq!(
        dry_run.snapshot.readiness,
        RecoveryReadiness::CorruptCheckpoint
    );
    assert!(dry_run.blocked);
    assert!(dry_run.snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == RecoveryDiagnosticSeverity::Error
            && diagnostic.code == "recovery.checkpoint.corrupt"
    }));
}

#[test]
fn recovery_snapshot_filters_terminal_and_idempotent_tool_calls() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(conversation_id, turn_id, "before checkpoint"))
        .expect("append first");
    store
        .append(tool_call_event(
            conversation_id,
            turn_id,
            "call-pending",
            "workspace.edit_file",
        ))
        .expect("append pending");
    store
        .append(tool_call_event(
            conversation_id,
            turn_id,
            "call-complete",
            "workspace.read_file",
        ))
        .expect("append complete proposal");
    store
        .append(tool_completed_event(
            conversation_id,
            turn_id,
            "call-complete",
        ))
        .expect("append complete terminal");
    store
        .append(tool_call_event(
            conversation_id,
            turn_id,
            "call-effect",
            "workspace.write_file",
        ))
        .expect("append effect proposal");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "state": "after checkpoint" }),
        ))
        .expect("checkpoint");
    record_effect(&mut store, "call-effect", "workspace.write_file");

    let snapshot = RecoveryEngine.snapshot(&store, &store, conversation_id, Some(turn_id));
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert_eq!(snapshot.readiness, RecoveryReadiness::PendingToolCalls);
    assert_eq!(snapshot.pending_tool_calls.len(), 1);
    assert_eq!(snapshot.pending_tool_calls[0].call_id, "call-pending");
    assert_eq!(
        snapshot.pending_tool_calls[0].status,
        RecoveredToolCallStatus::Pending
    );
    assert_eq!(snapshot.idempotent_effects.len(), 1);
    assert_eq!(snapshot.idempotent_effects[0].call_id, "call-effect");
    assert_eq!(
        snapshot.idempotent_effects[0].tool_name,
        "workspace.write_file"
    );
    assert!(!serialized.contains("must-not-appear"));
}

#[test]
fn recovery_snapshot_reports_unresolved_approval_waits() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();
    let checkpoint_id = CheckpointId::new();

    store
        .append(approval_requested_event(
            conversation_id,
            turn_id,
            Some(checkpoint_id),
        ))
        .expect("append approval");

    let snapshot = RecoveryEngine.snapshot(&store, &store, conversation_id, Some(turn_id));

    assert_eq!(snapshot.readiness, RecoveryReadiness::NeedsApproval);
    assert_eq!(snapshot.pending_approvals.len(), 1);
    assert_eq!(
        snapshot.pending_approvals[0].checkpoint_id,
        Some(checkpoint_id)
    );
    assert_eq!(snapshot.pending_approvals[0].action, "write");
    assert_eq!(
        snapshot.pending_approvals[0].resource,
        "workspace.edit_file"
    );
}

#[test]
fn resume_dry_run_reports_counts_without_side_effects() {
    let mut store = InMemoryEventStore::default();
    let conversation_id = ConversationId::new();
    let turn_id = TurnId::new();

    store
        .append(user_event(conversation_id, turn_id, "before checkpoint"))
        .expect("append first");
    store
        .append(tool_call_event(
            conversation_id,
            turn_id,
            "call-pending",
            "workspace.edit_file",
        ))
        .expect("append pending");
    store
        .append(tool_call_event(
            conversation_id,
            turn_id,
            "call-effect",
            "workspace.write_file",
        ))
        .expect("append effect proposal");
    store
        .put_checkpoint(checkpoint(
            CheckpointId::new(),
            conversation_id,
            turn_id,
            1,
            json!({ "state": "after checkpoint" }),
        ))
        .expect("checkpoint");
    record_effect(&mut store, "call-effect", "workspace.write_file");

    let dry_run = RecoveryEngine.resume_dry_run(&store, &store, conversation_id, Some(turn_id));

    assert!(!dry_run.blocked);
    assert_eq!(dry_run.will_replay_event_count, 2);
    assert_eq!(dry_run.will_resume_pending_tool_count, 1);
    assert_eq!(dry_run.will_reuse_idempotent_effect_count, 1);
    assert_eq!(
        store
            .get_effect(&IdempotencyKey::new("call-effect").expect("key"))
            .expect("effect")
            .expect("record")
            .call_id,
        "call-effect"
    );
}
