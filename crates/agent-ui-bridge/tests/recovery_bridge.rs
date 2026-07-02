use agent_types::{
    CheckpointId, ClientKind, ConversationId, CorrelationId, EventSeq, RecoveryReplaySummary,
    RecoverySnapshot, ResumeDryRunReport, SchemaVersion, TurnId,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame, UiRecoverySnapshot,
    UiReplayForkReport, UiReplayInspectReport, UiResumeDryRunReport, validate_ui_command,
};

fn conversation_id(value: &str) -> ConversationId {
    value.parse().expect("valid conversation id")
}

fn turn_id(value: &str) -> TurnId {
    value.parse().expect("valid turn id")
}

fn checkpoint_id(value: &str) -> CheckpointId {
    value.parse().expect("valid checkpoint id")
}

fn snapshot(conversation_id: ConversationId, turn_id: Option<TurnId>) -> RecoverySnapshot {
    RecoverySnapshot {
        conversation_id,
        turn_id,
        checkpoint: None,
        replay: RecoveryReplaySummary {
            replay_from: EventSeq::new(1).expect("event seq"),
            replayed_event_count: 0,
        },
        readiness: agent_types::RecoveryReadiness::Ready,
        pending_tool_calls: Vec::new(),
        idempotent_effects: Vec::new(),
        pending_approvals: Vec::new(),
        diagnostics: Vec::new(),
    }
}

#[test]
fn recovery_command_serializes_stable_shape() {
    let conversation_id = conversation_id("00000000-0000-0000-0000-000000006211");
    let turn_id = turn_id("00000000-0000-0000-0000-000000006212");
    let envelope = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "test".to_string(),
        },
        command: UiCommand::RecoveryResumeDryRun {
            conversation_id,
            turn_id: Some(turn_id),
            checkpoint_id: Some(checkpoint_id("00000000-0000-0000-0000-000000006213")),
        },
    };

    validate_ui_command(&envelope).expect("valid recovery command");
    let value = serde_json::to_value(&envelope.command).expect("serialize command");

    assert_eq!(value["type"], "recovery_resume_dry_run");
    assert_eq!(
        value["data"]["conversation_id"],
        conversation_id.to_string()
    );
    assert_eq!(value["data"]["turn_id"], turn_id.to_string());
}

#[test]
fn replay_fork_rejects_empty_label_and_nil_conversation() {
    let nil_conversation_id = conversation_id("00000000-0000-0000-0000-000000000000");
    let envelope = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Test,
            version: "test".to_string(),
        },
        command: UiCommand::ReplayFork {
            conversation_id: nil_conversation_id,
            source_turn_id: None,
            checkpoint_id: None,
            fork_label: " ".to_string(),
        },
    };

    let error = validate_ui_command(&envelope).expect_err("invalid replay fork command");
    assert!(error.to_string().contains("conversation_id cannot be nil"));

    let envelope = UiCommandEnvelope {
        command: UiCommand::ReplayFork {
            conversation_id: conversation_id("00000000-0000-0000-0000-000000006214"),
            source_turn_id: None,
            checkpoint_id: None,
            fork_label: " ".to_string(),
        },
        ..envelope
    };

    let error = validate_ui_command(&envelope).expect_err("empty label is invalid");
    assert!(error.to_string().contains("fork_label cannot be empty"));
}

#[test]
fn recovery_frames_serialize_typed_payloads() {
    let conversation_id = conversation_id("00000000-0000-0000-0000-000000006215");
    let turn_id = turn_id("00000000-0000-0000-0000-000000006216");
    let snapshot = snapshot(conversation_id, Some(turn_id));

    let recovery_frame = UiJsonFrame::RecoverySnapshot(UiRecoverySnapshot::new(snapshot.clone()));
    let recovery_value = serde_json::to_value(&recovery_frame).expect("serialize frame");
    assert_eq!(recovery_value["type"], "recovery_snapshot");
    assert_eq!(
        recovery_value["data"]["snapshot"]["conversation_id"],
        conversation_id.to_string()
    );

    let dry_run = ResumeDryRunReport {
        snapshot,
        will_replay_event_count: 2,
        will_resume_pending_tool_count: 1,
        will_reuse_idempotent_effect_count: 1,
        blocked: false,
    };
    let dry_run_frame = UiJsonFrame::ResumeDryRun(UiResumeDryRunReport::new(dry_run));
    let dry_run_value = serde_json::to_value(&dry_run_frame).expect("serialize dry run frame");
    assert_eq!(dry_run_value["type"], "resume_dry_run");
    assert_eq!(
        dry_run_value["data"]["report"]["will_resume_pending_tool_count"],
        1
    );
}

#[test]
fn replay_reports_are_bounded_and_redacted() {
    let conversation_id = conversation_id("00000000-0000-0000-0000-000000006217");
    let checkpoint_id = checkpoint_id("00000000-0000-0000-0000-000000006218");
    let replay_from = EventSeq::new(2).expect("event seq");

    let inspect = UiReplayInspectReport::new(
        conversation_id,
        replay_from,
        4,
        Some(checkpoint_id),
        Vec::new(),
    );
    let inspect_value =
        serde_json::to_value(UiJsonFrame::ReplayInspect(inspect)).expect("serialize inspect");
    assert_eq!(inspect_value["type"], "replay_inspect");
    assert_eq!(inspect_value["data"]["replayed_event_count"], 4);

    let fork = UiReplayForkReport::new(
        conversation_id,
        None,
        Some(checkpoint_id),
        "branch sk-secret-token",
        replay_from,
        true,
        Vec::new(),
    )
    .expect("valid fork report");
    let fork_json =
        serde_json::to_string(&UiJsonFrame::ReplayFork(fork)).expect("serialize fork report");
    assert!(fork_json.contains("replay_fork"));
    assert!(!fork_json.contains("sk-secret-token"));
    assert!(!fork_json.contains("api_key"));
}
