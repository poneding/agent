use ruse_cli::{
    CliCommand, OutputMode, PlanTodoCliCommand, PlanTodoCommandRuntime, parse_cli_command,
    parse_cli_invocation, render_cli_invocation, render_plan_todo_command,
};
use ruse_types::{
    PlanTodoItem, PlanTodoItemId, PlanTodoSnapshot, TodoPriority, TodoSource, TodoStatus,
};

#[test]
fn plan_todo_commands_parse_plan_and_todo_subcommands() {
    assert_eq!(
        parse_cli_command(["plan", "show"]).expect("parse plan show"),
        CliCommand::Plan(PlanTodoCliCommand::Show)
    );
    assert_eq!(
        parse_cli_command(["todo", "list"]).expect("parse todo list"),
        CliCommand::Todo(PlanTodoCliCommand::List)
    );
    assert_eq!(
        parse_cli_command([
            "todo",
            "update",
            "--todo",
            "todo-1",
            "--status",
            "in_progress",
            "--revision",
            "3",
            "--reason",
            "started",
        ])
        .expect("parse update"),
        CliCommand::Todo(PlanTodoCliCommand::UpdateStatus {
            item_id: "todo-1".to_string(),
            status: TodoStatus::InProgress,
            expected_revision: 3,
            reason: "started".to_string(),
        })
    );
}

#[test]
fn plan_todo_commands_render_text_plan_against_fixture_runtime() {
    let mut runtime = PlanTodoCommandRuntime::fixture();

    let output = render_plan_todo_command(&mut runtime, PlanTodoCliCommand::Show, OutputMode::Text);

    assert_eq!(output.status_code, 0);
    assert!(
        output
            .stdout
            .contains("Plan 00000000-0000-0000-0000-000000006102")
    );
    assert!(output.stdout.contains("revision=3"));
    assert!(
        output
            .stdout
            .contains("progress total=3 pending=1 in_progress=1 completed=1 cancelled=0")
    );
    assert!(output.stdout.contains("todo-1 status=pending"));
    assert!(output.stdout.contains("[redacted]"));
    assert!(!output.stdout.contains("sk-secret-token"));
}

#[test]
fn plan_todo_commands_render_json_snapshot() {
    let invocation =
        parse_cli_invocation(["--json", "plan", "show"]).expect("parse json plan show");
    assert_eq!(invocation.output_mode, OutputMode::Json);

    let output = render_cli_invocation(invocation);

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("\"type\":\"plan_todo_snapshot\""));
    assert!(output.stdout.contains("\"revision\":3"));
    assert!(output.stdout.contains("\"in_progress\":1"));
    assert!(!output.stdout.contains("sk-secret-token"));
}

#[test]
fn plan_todo_commands_update_status_with_revision() {
    let mut runtime = PlanTodoCommandRuntime::fixture();

    let output = render_plan_todo_command(
        &mut runtime,
        PlanTodoCliCommand::UpdateStatus {
            item_id: "todo-1".to_string(),
            status: TodoStatus::Completed,
            expected_revision: 3,
            reason: "finished first pass with token sk-secret-token".to_string(),
        },
        OutputMode::Text,
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Updated todo revision 3 -> 4"));
    assert!(output.stdout.contains("todo-1 status=completed"));
    assert!(
        output
            .stdout
            .contains("reason=finished first pass with [redacted] [redacted]")
    );
    assert_eq!(runtime.snapshot().revision, 4);
}

#[test]
fn plan_todo_commands_report_stale_revision_as_json_error() {
    let mut runtime = PlanTodoCommandRuntime::fixture();

    let output = render_plan_todo_command(
        &mut runtime,
        PlanTodoCliCommand::UpdateStatus {
            item_id: "todo-1".to_string(),
            status: TodoStatus::Completed,
            expected_revision: 99,
            reason: "bad revision".to_string(),
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("\"type\":\"plan_todo_snapshot\""));
    assert!(
        output
            .stdout
            .contains("\"code\":\"plan_todo_command_failed\"")
    );
    assert!(output.stdout.contains("stale plan/todo revision"));
    assert!(output.stdout.contains("\"items\":[]"));
}

#[test]
fn plan_todo_commands_reject_terminal_status_changes() {
    let mut runtime = PlanTodoCommandRuntime::fixture();

    let output = render_plan_todo_command(
        &mut runtime,
        PlanTodoCliCommand::UpdateStatus {
            item_id: "todo-3".to_string(),
            status: TodoStatus::Pending,
            expected_revision: 3,
            reason: "reopen completed work".to_string(),
        },
        OutputMode::Text,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("terminal todo item todo-3"));
}

#[test]
fn plan_todo_commands_report_missing_todo_item() {
    let mut runtime = PlanTodoCommandRuntime::fixture();

    let output = render_plan_todo_command(
        &mut runtime,
        PlanTodoCliCommand::UpdateStatus {
            item_id: "todo-missing".to_string(),
            status: TodoStatus::Completed,
            expected_revision: 3,
            reason: "complete missing item".to_string(),
        },
        OutputMode::Json,
    );

    assert_eq!(output.status_code, 2);
    assert!(output.stdout.contains("todo item not found: todo-missing"));
    assert!(output.stdout.contains("\"items\":[]"));
}

#[test]
fn plan_todo_commands_replace_snapshot_items() {
    let mut runtime = PlanTodoCommandRuntime::fixture();
    let mut replacement: PlanTodoSnapshot = runtime.snapshot().clone();
    replacement.items = vec![PlanTodoItem {
        item_id: PlanTodoItemId::from_string_unchecked("todo-new"),
        content: "Write follow-up docs".to_string(),
        status: TodoStatus::Pending,
        priority: TodoPriority::High,
        source: TodoSource::User,
        task_id: None,
        evidence: Vec::new(),
        updated_at: replacement.updated_at,
    }];
    let snapshot_json = serde_json::to_string(&replacement).expect("snapshot json");

    let output = render_plan_todo_command(
        &mut runtime,
        PlanTodoCliCommand::ReplaceSnapshot {
            snapshot_json,
            expected_revision: 3,
            reason: "replace from typed snapshot".to_string(),
        },
        OutputMode::Text,
    );

    assert_eq!(output.status_code, 0);
    assert!(output.stdout.contains("Replaced todos revision 3 -> 4"));
    assert!(output.stdout.contains("todo-new status=pending"));
    assert!(!output.stdout.contains("todo-2 status=in_progress"));
}
