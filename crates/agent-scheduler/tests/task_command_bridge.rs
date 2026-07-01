use agent_scheduler::{
    InMemoryTaskCommandService, TaskCommand, TaskCommandResponse, TaskCreateRequest,
    TaskNotificationKind, TaskPriority, TaskSchedule, TaskScheduleUpdate,
};
use agent_types::{ConversationId, TaskState, Timestamp, WorkspaceId};
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

fn create_request(prompt: &str) -> TaskCreateRequest {
    TaskCreateRequest::new(
        workspace_id(),
        Some(conversation_id()),
        prompt,
        TaskSchedule::OneShot { run_at: None },
        TaskPriority::Normal,
    )
    .expect("valid create request")
}

#[test]
fn task_command_bridge_creates_lists_and_inspects_tasks() {
    let mut service = InMemoryTaskCommandService::default();

    let created = service
        .handle(
            TaskCommand::Create(create_request("Review the workspace")),
            ts(1),
        )
        .expect("create task");
    let TaskCommandResponse::Created(created) = created else {
        panic!("expected created response");
    };

    assert_eq!(created.workspace_id, workspace_id());
    assert_eq!(created.conversation_id, Some(conversation_id()));
    assert_eq!(created.prompt, "Review the workspace");
    assert_eq!(created.schedule, TaskSchedule::OneShot { run_at: None });
    assert_eq!(created.state, TaskState::Queued);
    assert_eq!(created.created_at, ts(1));
    assert_eq!(created.updated_at, ts(1));
    assert_eq!(
        created
            .latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Queued)
    );

    let listed = service
        .handle(TaskCommand::List, ts(2))
        .expect("list tasks");
    let TaskCommandResponse::Listed(listed) = listed else {
        panic!("expected listed response");
    };
    assert_eq!(listed, vec![created.clone()]);

    let inspected = service
        .handle(
            TaskCommand::Inspect {
                task_id: created.task_id,
            },
            ts(3),
        )
        .expect("inspect task");
    assert_eq!(inspected, TaskCommandResponse::Inspected(created));
}

#[test]
fn task_command_bridge_updates_pause_resume_and_cancel_state() {
    let mut service = InMemoryTaskCommandService::default();
    let TaskCommandResponse::Created(task) = service
        .handle(
            TaskCommand::Create(create_request("Run nightly checks")),
            ts(1),
        )
        .expect("create")
    else {
        panic!("expected created response");
    };

    let TaskCommandResponse::ScheduleUpdated(updated) = service
        .handle(
            TaskCommand::UpdateSchedule(
                TaskScheduleUpdate::new(
                    task.task_id,
                    TaskSchedule::Cron {
                        expression: "0 8 * * *".to_string(),
                    },
                    "move to morning run",
                )
                .expect("schedule update"),
            ),
            ts(2),
        )
        .expect("update schedule")
    else {
        panic!("expected schedule update");
    };
    assert_eq!(
        updated.schedule,
        TaskSchedule::Cron {
            expression: "0 8 * * *".to_string()
        }
    );
    assert_eq!(updated.updated_at, ts(2));

    let TaskCommandResponse::Paused(paused) = service
        .handle(
            TaskCommand::Pause {
                task_id: task.task_id,
                reason: "hold for review".to_string(),
            },
            ts(3),
        )
        .expect("pause")
    else {
        panic!("expected paused");
    };
    assert_eq!(paused.state, TaskState::Paused);
    assert_eq!(
        paused
            .latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Paused)
    );
    assert_eq!(
        paused
            .latest_notification
            .as_ref()
            .map(|notification| notification.summary.as_str()),
        Some("hold for review")
    );

    let TaskCommandResponse::Resumed(resumed) = service
        .handle(
            TaskCommand::Resume {
                task_id: task.task_id,
                reason: "continue".to_string(),
            },
            ts(4),
        )
        .expect("resume")
    else {
        panic!("expected resumed");
    };
    assert_eq!(resumed.state, TaskState::Queued);
    assert_eq!(
        resumed
            .latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Resumed)
    );

    let TaskCommandResponse::Cancelled(cancelled) = service
        .handle(
            TaskCommand::Cancel {
                task_id: task.task_id,
                reason: "user stopped it".to_string(),
            },
            ts(5),
        )
        .expect("cancel")
    else {
        panic!("expected cancelled");
    };
    assert_eq!(cancelled.state, TaskState::Cancelled);
    assert_eq!(
        cancelled
            .latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Cancelled)
    );
}

#[test]
fn task_command_bridge_records_control_evidence_redacted() {
    let mut service = InMemoryTaskCommandService::default();
    let TaskCommandResponse::Created(task) = service
        .handle(
            TaskCommand::Create(create_request("Run nightly checks")),
            ts(1),
        )
        .expect("create")
    else {
        panic!("expected created response");
    };

    let TaskCommandResponse::Paused(paused) = service
        .handle(
            TaskCommand::Pause {
                task_id: task.task_id,
                reason: "hold because api_key leaked".to_string(),
            },
            ts(2),
        )
        .expect("pause")
    else {
        panic!("expected paused");
    };

    let notification = paused
        .latest_notification
        .as_ref()
        .expect("latest notification");
    assert_eq!(notification.kind, TaskNotificationKind::Paused);
    assert_eq!(notification.summary, "hold because [redacted] leaked");
    assert!(!notification.summary.contains("api_key"));
}

#[test]
fn task_command_bridge_validates_inputs_and_terminal_updates() {
    let mut service = InMemoryTaskCommandService::default();

    let empty = TaskCreateRequest::new(
        workspace_id(),
        None,
        " ",
        TaskSchedule::OneShot { run_at: None },
        TaskPriority::Low,
    );
    assert!(empty.is_err());

    let TaskCommandResponse::Created(task) = service
        .handle(
            TaskCommand::Create(create_request("Stop after creation")),
            ts(1),
        )
        .expect("create")
    else {
        panic!("expected created");
    };
    service
        .handle(
            TaskCommand::Cancel {
                task_id: task.task_id,
                reason: "done".to_string(),
            },
            ts(2),
        )
        .expect("cancel");

    let err = service
        .handle(
            TaskCommand::UpdateSchedule(
                TaskScheduleUpdate::new(
                    task.task_id,
                    TaskSchedule::OneShot {
                        run_at: Some(ts(5)),
                    },
                    "change terminal task",
                )
                .expect("valid update"),
            ),
            ts(3),
        )
        .expect_err("terminal task cannot update schedule");
    assert!(err.to_string().contains("terminal"));
}
