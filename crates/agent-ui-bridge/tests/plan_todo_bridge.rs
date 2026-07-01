use agent_types::{
    ClientKind, ConversationId, CorrelationId, PlanTodoDiagnostic, PlanTodoEvidence,
    PlanTodoEvidenceKind, PlanTodoItem, PlanTodoItemId, PlanTodoSnapshot, SchemaVersion, Timestamp,
    TodoPriority, TodoSource, TodoStatus,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame, UiPlanTodoCommand,
    UiPlanTodoCommandError, UiPlanTodoSnapshot, validate_ui_command,
};
use time::macros::datetime;

fn ts(second: u8) -> Timestamp {
    Timestamp::from_utc(datetime!(2026-07-02 00:00:00 UTC) + time::Duration::seconds(second.into()))
}

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000006102"
        .parse()
        .expect("conversation id")
}

fn item(id: &str, content: &str, status: TodoStatus) -> PlanTodoItem {
    PlanTodoItem {
        item_id: PlanTodoItemId::from_string_unchecked(id),
        content: content.to_string(),
        status,
        priority: TodoPriority::Normal,
        source: TodoSource::Model,
        task_id: None,
        evidence: Vec::new(),
        updated_at: ts(1),
    }
}

fn snapshot(items: Vec<PlanTodoItem>) -> PlanTodoSnapshot {
    PlanTodoSnapshot {
        conversation_id: conversation_id(),
        turn_id: None,
        task_id: None,
        revision: 7,
        items,
        updated_at: ts(2),
        diagnostics: Vec::new(),
    }
}

#[test]
fn plan_todo_bridge_serializes_command_envelope() {
    let command = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::PlanTodo(UiPlanTodoCommand::UpdateStatus {
            conversation_id: conversation_id(),
            item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
            status: TodoStatus::InProgress,
            expected_revision: 7,
            reason: "started implementation".to_string(),
        }),
    };

    validate_ui_command(&command).expect("valid command");
    let value = serde_json::to_value(command).expect("serialize");

    assert_eq!(value["command"]["type"], "plan_todo");
    assert_eq!(value["command"]["data"]["type"], "update_status");
    assert_eq!(value["command"]["data"]["data"]["status"], "in_progress");
}

#[test]
fn plan_todo_bridge_rejects_empty_update_reason() {
    let command = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::PlanTodo(UiPlanTodoCommand::UpdateStatus {
            conversation_id: conversation_id(),
            item_id: PlanTodoItemId::from_string_unchecked("todo-1"),
            status: TodoStatus::Completed,
            expected_revision: 7,
            reason: "  ".to_string(),
        }),
    };

    let err = validate_ui_command(&command).expect_err("empty reason rejected");
    assert!(err.to_string().contains("plan_todo reason cannot be empty"));
}

#[test]
fn plan_todo_bridge_snapshot_is_sorted_redacted_and_progressed() {
    let mut first = item(
        "todo-b",
        "Run tests with sk-secret-token before final review",
        TodoStatus::Completed,
    );
    first.evidence.push(PlanTodoEvidence {
        kind: PlanTodoEvidenceKind::Artifact,
        reference: "artifact-secret".to_string(),
        summary: "contains API_TOKEN=abc".to_string(),
    });

    let mut plan = snapshot(vec![
        first,
        item("todo-a", "Inspect code", TodoStatus::Pending),
    ]);
    plan.diagnostics.push(PlanTodoDiagnostic {
        code: "note".to_string(),
        message: "token leaked in model text".to_string(),
    });

    let ui = UiPlanTodoSnapshot::from_snapshot(plan, Vec::new()).expect("snapshot");

    assert_eq!(ui.progress.total, 2);
    assert_eq!(ui.progress.pending, 1);
    assert_eq!(ui.progress.completed, 1);
    assert_eq!(
        ui.snapshot
            .items
            .iter()
            .map(|item| item.item_id.to_string())
            .collect::<Vec<_>>(),
        vec!["todo-a", "todo-b"]
    );

    let serialized = serde_json::to_string(&UiJsonFrame::PlanTodoSnapshot(ui)).expect("serialize");
    assert!(serialized.contains("\"type\":\"plan_todo_snapshot\""));
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("API_TOKEN=abc"));
}

#[test]
fn plan_todo_bridge_empty_snapshot_is_successful() {
    let ui = UiPlanTodoSnapshot::from_snapshot(snapshot(Vec::new()), Vec::new()).expect("empty");

    assert!(ui.snapshot.items.is_empty());
    assert_eq!(ui.progress.total, 0);
    assert!(ui.errors.is_empty());
}

#[test]
fn plan_todo_bridge_errors_are_bounded_and_redacted() {
    let error = UiPlanTodoCommandError::recoverable(
        "plan_todo_failed",
        "failed because api_key sk-secret-token",
    )
    .expect("error");
    let ui =
        UiPlanTodoSnapshot::from_snapshot(snapshot(Vec::new()), vec![error]).expect("snapshot");

    assert_eq!(ui.errors[0].message, "failed because [redacted] [redacted]");
}
