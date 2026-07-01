use agent_core::{
    PlanTodoError, PlanTodoOperation, PlanTodoUpdate, apply_plan_todo_update,
    normalize_plan_todo_snapshot, validate_plan_todo_snapshot,
};
use agent_types::{
    ConversationId, PlanTodoDiagnostic, PlanTodoEvidence, PlanTodoEvidenceKind, PlanTodoItem,
    PlanTodoItemId, PlanTodoSnapshot, Timestamp, TodoPriority, TodoSource, TodoStatus, TurnId,
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
