use agent_scheduler::{
    BackgroundAgentTask, TaskCommand, TaskCreateRequest, TaskPriority, TaskSchedule,
    TaskScheduleUpdate, redact_task_text,
};
use agent_types::{
    ClientKind, ConversationId, CorrelationId, SchemaVersion, TaskId, TaskState, Timestamp,
    WorkspaceId,
};
use agent_ui_bridge::{
    UiClientInfo, UiCommand, UiCommandEnvelope, UiJsonFrame, UiTaskCommandError, UiTaskSnapshot,
    UiTaskSummary, validate_ui_command,
};
use time::macros::datetime;

fn ts(second: u8) -> Timestamp {
    Timestamp::from_utc(datetime!(2026-06-27 00:00:00 UTC) + time::Duration::seconds(second.into()))
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000002401"
        .parse()
        .expect("workspace id")
}

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000002411"
        .parse()
        .expect("conversation id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000002421"
        .parse()
        .expect("task id")
}

fn task(prompt: &str) -> BackgroundAgentTask {
    BackgroundAgentTask {
        task_id: task_id(),
        workspace_id: workspace_id(),
        conversation_id: Some(conversation_id()),
        prompt: prompt.to_string(),
        schedule: TaskSchedule::Cron {
            expression: "0 8 * * *".to_string(),
        },
        state: TaskState::Queued,
        priority: TaskPriority::High,
        created_at: ts(1),
        updated_at: ts(2),
        latest_run_summary: Some("waiting for next lease".to_string()),
    }
}

#[test]
fn task_command_bridge_serializes_ui_safe_task_snapshot() {
    let summary = UiTaskSummary::from_task(&task(
        "Review project with sk-secret-token and API_TOKEN=abc before release",
    ))
    .expect("summary");
    let snapshot = UiTaskSnapshot::new(vec![summary], Vec::new()).expect("snapshot");
    let serialized =
        serde_json::to_string(&UiJsonFrame::TaskSnapshot(snapshot)).expect("serialize");

    assert!(serialized.contains("\"type\":\"task_snapshot\""));
    assert!(serialized.contains("\"state\":\"queued\""));
    assert!(serialized.contains("\"priority\":\"high\""));
    assert!(serialized.contains("\"kind\":\"cron\""));
    assert!(serialized.contains("0 8 * * *"));
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("API_TOKEN=abc"));
}

#[test]
fn task_command_bridge_sorts_summaries_and_redacts_errors() {
    let first = UiTaskSummary::from_task(&task("first task")).expect("first");
    let mut second_task = task("second task");
    second_task.task_id = "00000000-0000-0000-0000-000000002420"
        .parse()
        .expect("task id");
    let second = UiTaskSummary::from_task(&second_task).expect("second");
    let error =
        UiTaskCommandError::recoverable("missing_task", "task uses password secret").expect("err");

    let snapshot = UiTaskSnapshot::new(vec![first, second], vec![error]).expect("snapshot");

    assert_eq!(
        snapshot
            .tasks
            .iter()
            .map(|task| task.task_id.to_string())
            .collect::<Vec<_>>(),
        vec![
            "00000000-0000-0000-0000-000000002420",
            "00000000-0000-0000-0000-000000002421",
        ]
    );
    assert_eq!(
        snapshot.errors[0].message,
        "task uses [redacted] [redacted]"
    );
}

#[test]
fn task_command_bridge_preserves_daily_driver_task_states() {
    let states = [
        TaskState::Queued,
        TaskState::Running,
        TaskState::WaitingApproval,
        TaskState::Paused,
        TaskState::Succeeded,
        TaskState::Failed,
        TaskState::Cancelled,
    ];
    let summaries = states
        .into_iter()
        .enumerate()
        .map(|(index, state)| {
            let mut task = task(&format!("task state {state:?}"));
            task.task_id = format!("00000000-0000-0000-0000-00000000243{index}")
                .parse()
                .expect("task id");
            task.state = state;
            UiTaskSummary::from_task(&task).expect("summary")
        })
        .collect::<Vec<_>>();

    let snapshot = UiTaskSnapshot::new(summaries, Vec::new()).expect("snapshot");
    let serialized = serde_json::to_string(&snapshot).expect("serialize");

    assert!(serialized.contains("\"state\":\"queued\""));
    assert!(serialized.contains("\"state\":\"running\""));
    assert!(serialized.contains("\"state\":\"waiting_approval\""));
    assert!(serialized.contains("\"state\":\"paused\""));
    assert!(serialized.contains("\"state\":\"succeeded\""));
    assert!(serialized.contains("\"state\":\"failed\""));
    assert!(serialized.contains("\"state\":\"cancelled\""));
}

#[test]
fn task_command_bridge_empty_snapshot_is_successful() {
    let snapshot = UiTaskSnapshot::new(Vec::new(), Vec::new()).expect("snapshot");

    assert!(snapshot.tasks.is_empty());
    assert!(snapshot.errors.is_empty());
}

#[test]
fn task_command_bridge_redaction_helper_is_shared_with_scheduler() {
    let update = TaskScheduleUpdate::new(
        task_id(),
        TaskSchedule::OneShot {
            run_at: Some(ts(8)),
        },
        "move because api_key leaked",
    )
    .expect("update");

    assert_eq!(update.reason, "move because [redacted] leaked");
    assert_eq!(redact_task_text("token sk-secret"), "[redacted] [redacted]");
}

#[test]
fn task_command_bridge_serializes_ui_task_command() {
    let command = UiCommandEnvelope {
        schema_version: SchemaVersion::current(),
        command_id: CorrelationId::new(),
        client: UiClientInfo {
            kind: ClientKind::Desktop,
            version: "0.1.0".to_string(),
        },
        command: UiCommand::Task(TaskCommand::Create(
            TaskCreateRequest::new(
                workspace_id(),
                Some(conversation_id()),
                "run background review",
                TaskSchedule::OneShot { run_at: None },
                TaskPriority::Normal,
            )
            .expect("create request"),
        )),
    };

    validate_ui_command(&command).expect("valid command");
    let value = serde_json::to_value(command).expect("serialize command");

    assert_eq!(value["command"]["type"], "task");
    assert_eq!(value["command"]["data"]["type"], "create");
    assert_eq!(
        value["command"]["data"]["data"]["prompt"],
        "run background review"
    );
}
