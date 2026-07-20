use agent_types::{
    CheckpointId, ClientKind, CommandRetryAttemptLink, CommandRetryCommandSummary,
    CommandRetryFailureClassification, CommandRetryPlan, CommandRetryReadiness, ConversationId,
    CorrelationId, EventSeq, IdempotentRecoverySummary, RecoveryConflictKind,
    RecoveryConflictNextAction, RecoveryConflictSide, RecoveryConflictSummary,
    RecoveryReplaySummary, RecoverySnapshot, ResumeDryRunReport, ResumeReport, SchemaVersion,
    TurnId,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiCommandRetryPlan, UiConflictInspectReport,
    UiIdempotentRecoverySummary, UiJsonFrame, UiRecoverySnapshot, UiReplayForkReport,
    UiReplayInspectReport, UiResumeDryRunReport, UiResumeReport, validate_ui_command,
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
        idempotent_recovery: agent_types::IdempotentRecoverySummary::default(),
        conflicts: Vec::new(),
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

    let recovery_frame = UiJsonFrame::recovery_snapshot(UiRecoverySnapshot::new(snapshot.clone()));
    let recovery_value = serde_json::to_value(&recovery_frame).expect("serialize frame");
    assert_eq!(recovery_value["type"], "recovery_snapshot");
    assert_eq!(
        recovery_value["data"]["snapshot"]["conversation_id"],
        conversation_id.to_string()
    );

    let dry_run = ResumeDryRunReport {
        snapshot: snapshot.clone(),
        will_replay_event_count: 2,
        will_resume_pending_tool_count: 1,
        will_reuse_idempotent_effect_count: 1,
        blocked: false,
    };
    let dry_run_frame = UiJsonFrame::resume_dry_run(UiResumeDryRunReport::new(dry_run));
    let dry_run_value = serde_json::to_value(&dry_run_frame).expect("serialize dry run frame");
    assert_eq!(dry_run_value["type"], "resume_dry_run");
    assert_eq!(
        dry_run_value["data"]["report"]["will_resume_pending_tool_count"],
        1
    );

    let resume = ResumeReport {
        snapshot,
        replayed_event_count: 2,
        resumed_pending_tool_count: 1,
        reused_idempotent_effect_count: 1,
        resumed: true,
        blocked: false,
        continuation: None,
    };
    let resume_frame = UiJsonFrame::recovery_resume(UiResumeReport::new(resume));
    let resume_value = serde_json::to_value(&resume_frame).expect("serialize resume frame");
    assert_eq!(resume_value["type"], "recovery_resume");
    assert_eq!(resume_value["data"]["report"]["resumed"], true);
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
        serde_json::to_value(UiJsonFrame::replay_inspect(inspect)).expect("serialize inspect");
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
        serde_json::to_string(&UiJsonFrame::replay_fork(fork)).expect("serialize fork report");
    assert!(fork_json.contains("replay_fork"));
    assert!(!fork_json.contains("sk-secret-token"));
    assert!(!fork_json.contains("api_key"));
}

#[test]
fn command_retry_command_validates_dry_run_shape() {
    let envelope = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Cli,
            version: "test".to_string(),
        },
        command: UiCommand::CommandRetryPlan {
            command_run_id: "command-run-1".to_string(),
            reason: "rerun after fix".to_string(),
            dry_run: true,
        },
    };

    validate_ui_command(&envelope).expect("valid command retry plan command");
    let value = serde_json::to_value(&envelope.command).expect("serialize command");
    assert_eq!(value["type"], "command_retry_plan");
    assert_eq!(value["data"]["command_run_id"], "command-run-1");

    let executable = UiCommandEnvelope {
        command: UiCommand::CommandRetryPlan {
            command_run_id: "command-run-1".to_string(),
            reason: "rerun".to_string(),
            dry_run: false,
        },
        ..envelope
    };
    validate_ui_command(&executable).expect("valid executable retry command");
}

#[test]
fn command_retry_and_idempotent_recovery_frames_serialize() {
    let plan = CommandRetryPlan {
        command_run_id: "command-run-1".to_string(),
        attempt: CommandRetryAttemptLink {
            previous_attempt_id: "command-run-1".to_string(),
            next_attempt_id: "command-run-1-retry-1".to_string(),
        },
        readiness: CommandRetryReadiness::Retryable,
        classification: CommandRetryFailureClassification::TestFailure,
        command: CommandRetryCommandSummary {
            display: "cargo test".to_string(),
            program: Some("cargo".to_string()),
            args: vec!["test".to_string()],
            timeout_ms: 30_000,
            cwd: ".".to_string(),
            classify_as_test: true,
        },
        exit_code: Some(101),
        reason: "rerun after [redacted]".to_string(),
        dry_run: true,
        diagnostics: Vec::new(),
    };
    let retry_json = serde_json::to_string(&UiJsonFrame::command_retry_plan(
        UiCommandRetryPlan::new(plan),
    ))
    .expect("serialize retry frame");
    assert!(retry_json.contains("command_retry_plan"));
    assert!(!retry_json.contains("sk-secret-token"));

    let idempotent_json = serde_json::to_string(&UiJsonFrame::idempotent_recovery(
        UiIdempotentRecoverySummary::new(IdempotentRecoverySummary {
            reusable_effect_count: 1,
            pending_tool_count: 0,
            conflict_count: 0,
            effects: Vec::new(),
            diagnostics: Vec::new(),
        }),
    ))
    .expect("serialize idempotent frame");
    assert!(idempotent_json.contains("idempotent_recovery"));
    assert!(idempotent_json.contains("reusable_effect_count"));
}

#[test]
fn conflict_inspect_frame_serializes_safe_shape() {
    let conflict = RecoveryConflictSummary {
        path: "src/lib.rs".to_string(),
        kind: RecoveryConflictKind::BothModified,
        lead_change: RecoveryConflictSide {
            source: "lead".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("lead-hash".to_string()),
        },
        incoming_change: RecoveryConflictSide {
            source: "incoming".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("incoming-hash".to_string()),
        },
        next_action: RecoveryConflictNextAction::InspectDiff,
        diagnostics: Vec::new(),
    };
    let frame =
        UiJsonFrame::conflict_inspect(UiConflictInspectReport::new(vec![conflict], Vec::new()));
    let value = serde_json::to_value(frame).expect("serialize conflict frame");

    assert_eq!(value["type"], "conflict_inspect");
    assert_eq!(value["data"]["conflicts"][0]["path"], "src/lib.rs");
    assert_eq!(value["data"]["conflicts"][0]["kind"], "both_modified");
    assert_eq!(value["data"]["conflicts"][0]["next_action"], "inspect_diff");
}
