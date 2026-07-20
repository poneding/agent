use ruse_core::{
    PlanTodoError, PlanTodoOperation, PlanTodoUpdate, TaskArtifact, TaskArtifactKind,
    TaskReviewSummary, append_task_review_evidence, apply_plan_todo_update,
    normalize_plan_todo_snapshot, validate_plan_todo_snapshot,
};
use ruse_types::{
    CheckpointId, ConversationId, PlanTodoDiagnostic, PlanTodoEvidence, PlanTodoEvidenceKind,
    PlanTodoItem, PlanTodoItemId, PlanTodoSnapshot, TaskId, TaskState, Timestamp, TodoPriority,
    TodoSource, TodoStatus, TurnId,
};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-01T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn todo(id: &str, content: &str, status: TodoStatus, second: u8) -> PlanTodoItem {
    PlanTodoItem {
        item_id: PlanTodoItemId::from_string_unchecked(id),
        content: content.to_string(),
        status,
        priority: TodoPriority::Normal,
        source: TodoSource::Model,
        task_id: None,
        evidence: Vec::new(),
        updated_at: ts(second),
    }
}

fn snapshot(items: Vec<PlanTodoItem>) -> PlanTodoSnapshot {
    PlanTodoSnapshot {
        conversation_id: ConversationId::new(),
        turn_id: Some(TurnId::new()),
        task_id: None,
        revision: 3,
        items,
        updated_at: ts(9),
        diagnostics: Vec::new(),
    }
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000006105"
        .parse()
        .expect("task id")
}

fn checkpoint_id() -> CheckpointId {
    "00000000-0000-0000-0000-000000006115"
        .parse()
        .expect("checkpoint id")
}

#[test]
fn plan_todo_snapshot_validates_and_counts_progress() {
    let snapshot = snapshot(vec![
        todo("todo-1", "Inspect task contracts", TodoStatus::Pending, 1),
        todo("todo-2", "Implement plan state", TodoStatus::InProgress, 2),
        todo("todo-3", "Write tests", TodoStatus::Completed, 3),
        todo(
            "todo-4",
            "Skip duplicate UI logic",
            TodoStatus::Cancelled,
            4,
        ),
    ]);

    let progress = validate_plan_todo_snapshot(&snapshot).expect("valid snapshot");

    assert_eq!(progress.total, 4);
    assert_eq!(progress.pending, 1);
    assert_eq!(progress.in_progress, 1);
    assert_eq!(progress.completed, 1);
    assert_eq!(progress.cancelled, 1);
}

#[test]
fn plan_todo_snapshot_allows_empty_lists() {
    let progress = validate_plan_todo_snapshot(&snapshot(Vec::new())).expect("empty is valid");

    assert_eq!(progress.total, 0);
    assert_eq!(progress.pending, 0);
}

#[test]
fn plan_todo_snapshot_rejects_invalid_items_and_diagnostics() {
    let empty_content = snapshot(vec![todo("todo-1", " ", TodoStatus::Pending, 1)]);
    assert!(matches!(
        validate_plan_todo_snapshot(&empty_content),
        Err(PlanTodoError::Validation(message)) if message.contains("todo content")
    ));

    let duplicate_ids = snapshot(vec![
        todo("todo-1", "first", TodoStatus::Pending, 1),
        todo("todo-1", "second", TodoStatus::Completed, 2),
    ]);
    assert!(matches!(
        validate_plan_todo_snapshot(&duplicate_ids),
        Err(PlanTodoError::Validation(message)) if message.contains("duplicate")
    ));

    let multiple_active = snapshot(vec![
        todo("todo-1", "first", TodoStatus::InProgress, 1),
        todo("todo-2", "second", TodoStatus::InProgress, 2),
    ]);
    assert!(matches!(
        validate_plan_todo_snapshot(&multiple_active),
        Err(PlanTodoError::Validation(message)) if message.contains("in_progress")
    ));

    let mut diagnostic = snapshot(Vec::new());
    diagnostic.diagnostics.push(PlanTodoDiagnostic {
        code: "diag.empty".to_string(),
        message: " ".to_string(),
    });
    assert!(matches!(
        validate_plan_todo_snapshot(&diagnostic),
        Err(PlanTodoError::Validation(message)) if message.contains("diagnostic message")
    ));
}

#[test]
fn plan_todo_normalization_redacts_and_bounds_text() {
    let mut item = todo(
        "todo-1",
        &format!("review sk-secret-token {}", "x".repeat(260)),
        TodoStatus::Pending,
        1,
    );
    item.evidence.push(PlanTodoEvidence {
        kind: PlanTodoEvidenceKind::Artifact,
        reference: "artifact-sk-secret-token".to_string(),
        summary: "created password-file output".to_string(),
    });
    let mut snapshot = snapshot(vec![item]);
    snapshot.diagnostics.push(PlanTodoDiagnostic {
        code: "diag.api_key".to_string(),
        message: "contains token-value".to_string(),
    });

    let (normalized, progress) = normalize_plan_todo_snapshot(snapshot).expect("normalize");

    assert_eq!(progress.pending, 1);
    assert!(normalized.items[0].content.chars().count() <= 240);
    assert!(normalized.items[0].content.contains("[redacted]"));
    assert!(!normalized.items[0].content.contains("sk-secret-token"));
    assert_eq!(
        normalized.items[0].evidence[0].summary,
        "created [redacted] output"
    );
    assert_eq!(normalized.diagnostics[0].code, "[redacted]");
    assert_eq!(normalized.diagnostics[0].message, "contains [redacted]");
}

#[test]
fn plan_todo_updates_check_revision_and_increment_on_success() {
    let snapshot = snapshot(vec![todo("todo-1", "Start", TodoStatus::Pending, 1)]);

    let outcome = apply_plan_todo_update(
        &snapshot,
        PlanTodoUpdate {
            expected_revision: 3,
            reason: "begin work".to_string(),
            operation: PlanTodoOperation::UpdateStatus {
                item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
                status: TodoStatus::InProgress,
            },
        },
    )
    .expect("update");

    assert_eq!(outcome.previous_revision, 3);
    assert_eq!(outcome.snapshot.revision, 4);
    assert_eq!(outcome.snapshot.items[0].status, TodoStatus::InProgress);
    assert_eq!(outcome.progress.in_progress, 1);
    assert_eq!(outcome.reason, "begin work");

    let stale = apply_plan_todo_update(
        &outcome.snapshot,
        PlanTodoUpdate {
            expected_revision: 3,
            reason: "stale update".to_string(),
            operation: PlanTodoOperation::UpdateStatus {
                item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
                status: TodoStatus::Completed,
            },
        },
    )
    .expect_err("stale revision");

    assert_eq!(
        stale,
        PlanTodoError::StaleRevision {
            expected: 3,
            actual: 4
        }
    );
}

#[test]
fn plan_todo_append_evidence_preserves_terminal_status() {
    let snapshot = snapshot(vec![todo("todo-1", "Finish", TodoStatus::Completed, 1)]);

    let outcome = apply_plan_todo_update(
        &snapshot,
        PlanTodoUpdate {
            expected_revision: 3,
            reason: "attach artifact sk-secret-token".to_string(),
            operation: PlanTodoOperation::AppendEvidence {
                item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
                evidence: PlanTodoEvidence {
                    kind: PlanTodoEvidenceKind::Artifact,
                    reference: "artifact-report".to_string(),
                    summary: "report with secret token".to_string(),
                },
            },
        },
    )
    .expect("append evidence");

    assert_eq!(outcome.snapshot.items[0].status, TodoStatus::Completed);
    assert_eq!(outcome.snapshot.items[0].evidence.len(), 1);
    assert_eq!(
        outcome.snapshot.items[0].evidence[0].summary,
        "report with [redacted] [redacted]"
    );
    assert_eq!(outcome.reason, "attach artifact [redacted]");

    let reopen = apply_plan_todo_update(
        &outcome.snapshot,
        PlanTodoUpdate {
            expected_revision: 4,
            reason: "reopen".to_string(),
            operation: PlanTodoOperation::UpdateStatus {
                item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
                status: TodoStatus::Pending,
            },
        },
    )
    .expect_err("terminal status");

    assert!(matches!(
        reopen,
        PlanTodoError::TerminalItemStatus { item_id, .. } if item_id == "todo-1"
    ));
}

#[test]
fn plan_todo_appends_task_review_artifact_evidence() {
    let mut item = todo("todo-1", "Finish task", TodoStatus::Completed, 1);
    item.task_id = None;
    let snapshot = snapshot(vec![item]);
    let artifact = TaskArtifact::new(
        "artifact-report",
        task_id(),
        TaskArtifactKind::Report,
        Some("artifacts/report.md"),
        Some("sha256:abc123"),
        "summary with sk-secret-token",
    )
    .expect("artifact");
    let missing = TaskArtifact::missing(
        "artifact-log",
        task_id(),
        TaskArtifactKind::Log,
        "missing log",
        "log had password secret",
    )
    .expect("missing");
    let review = TaskReviewSummary::from_artifacts(
        task_id(),
        TaskState::Succeeded,
        Some("trace-token-secret"),
        Some(checkpoint_id()),
        &[artifact.clone(), missing],
        "completed with api_key hidden",
    )
    .expect("review");

    let outcome = append_task_review_evidence(
        &snapshot,
        PlanTodoItemId::from_string_unchecked("todo-1"),
        3,
        &review,
        std::slice::from_ref(&artifact),
        "attach review token",
    )
    .expect("append review evidence");

    let item = &outcome.snapshot.items[0];
    assert_eq!(outcome.previous_revision, 3);
    assert_eq!(outcome.snapshot.revision, 4);
    assert_eq!(item.status, TodoStatus::Completed);
    assert_eq!(item.task_id, Some(task_id()));
    assert_eq!(outcome.reason, "attach review [redacted]");
    assert!(item.evidence.iter().any(|evidence| {
        evidence.kind == PlanTodoEvidenceKind::Task && evidence.reference == task_id().to_string()
    }));
    assert!(item.evidence.iter().any(|evidence| {
        evidence.kind == PlanTodoEvidenceKind::Trace && evidence.reference.contains("[redacted]")
    }));
    assert!(item.evidence.iter().any(|evidence| {
        evidence.kind == PlanTodoEvidenceKind::Checkpoint
            && evidence.reference == checkpoint_id().to_string()
    }));
    assert!(item.evidence.iter().any(|evidence| {
        evidence.kind == PlanTodoEvidenceKind::Artifact
            && evidence.reference == "artifact-report"
            && evidence.summary.contains("[redacted]")
    }));
    assert!(item.evidence.iter().any(|evidence| {
        evidence.kind == PlanTodoEvidenceKind::Diagnostic
            && evidence.summary == "missing artifact metadata: artifact-log"
    }));
    assert!(
        !serde_json::to_string(item)
            .expect("serialize")
            .contains("sk-secret-token")
    );
}

#[test]
fn plan_todo_task_review_evidence_checks_revision_and_item() {
    let snapshot = snapshot(vec![todo("todo-1", "Finish task", TodoStatus::Pending, 1)]);
    let review = TaskReviewSummary::new(
        task_id(),
        TaskState::Succeeded,
        None,
        None,
        Vec::new(),
        "done",
    )
    .expect("review");

    let stale = append_task_review_evidence(
        &snapshot,
        PlanTodoItemId::from_string_unchecked("todo-1"),
        2,
        &review,
        &[],
        "attach review",
    )
    .expect_err("stale revision");
    assert_eq!(
        stale,
        PlanTodoError::StaleRevision {
            expected: 2,
            actual: 3
        }
    );

    let missing = append_task_review_evidence(
        &snapshot,
        PlanTodoItemId::from_string_unchecked("missing"),
        3,
        &review,
        &[],
        "attach review",
    )
    .expect_err("missing item");
    assert!(matches!(missing, PlanTodoError::MissingItem(item) if item == "missing"));
}
