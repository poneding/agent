use agent_types::{ConversationId, TaskState, Timestamp, WorkspaceId};

fn task_run(summary: &str) -> agent_scheduler::TaskRun {
    agent_scheduler::TaskRun::new(
        WorkspaceId::new(),
        Some(ConversationId::new()),
        agent_scheduler::TaskKind::Background,
        agent_scheduler::TaskPriority::Normal,
        summary,
        Timestamp::now_utc(),
    )
    .unwrap()
}

fn transition(from: TaskState, to: TaskState, reason: &str) -> agent_scheduler::TaskTransition {
    agent_scheduler::TaskTransition {
        from,
        to,
        reason: reason.to_string(),
        at: Timestamp::now_utc(),
    }
}

#[test]
fn task_state_machine_serializes_task_run_shape() {
    let run = task_run("nightly index");

    let value = serde_json::to_value(&run).expect("serialize task run");

    assert_eq!(value["kind"], "background");
    assert_eq!(value["state"], "created");
    assert_eq!(value["priority"], "normal");
    assert_eq!(value["attempt"], 0);
    assert_eq!(value["max_attempts"], 1);
    assert_eq!(value["summary"], "nightly index");
}

#[test]
fn task_state_machine_allows_created_to_queued_to_running_to_succeeded() {
    let created = task_run("nightly index");
    let queued = agent_scheduler::transition_task_state(
        &created,
        transition(TaskState::Created, TaskState::Queued, "enqueue"),
    )
    .unwrap();
    let running = agent_scheduler::transition_task_state(
        &queued,
        transition(TaskState::Queued, TaskState::Running, "lease acquired"),
    )
    .unwrap();
    let succeeded = agent_scheduler::transition_task_state(
        &running,
        transition(TaskState::Running, TaskState::Succeeded, "completed"),
    )
    .unwrap();

    assert_eq!(queued.state, TaskState::Queued);
    assert_eq!(running.state, TaskState::Running);
    assert_eq!(succeeded.state, TaskState::Succeeded);
    assert_eq!(succeeded.attempt, 0);
}

#[test]
fn task_state_machine_rejects_invalid_transition() {
    let created = task_run("nightly index");

    let err = agent_scheduler::transition_task_state(
        &created,
        transition(TaskState::Created, TaskState::Running, "skip queue"),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        agent_scheduler::SchedulerError::InvalidTransition(_)
    ));
}

#[test]
fn task_state_machine_rejects_terminal_transition() {
    let mut succeeded = task_run("nightly index");
    succeeded.state = TaskState::Succeeded;

    let err = agent_scheduler::transition_task_state(
        &succeeded,
        transition(TaskState::Succeeded, TaskState::Queued, "retry"),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        agent_scheduler::SchedulerError::InvalidTransition(_)
    ));
}

#[test]
fn task_state_machine_rejects_empty_summary_and_reason() {
    let invalid_run = agent_scheduler::TaskRun::new(
        WorkspaceId::new(),
        None,
        agent_scheduler::TaskKind::OneShot,
        agent_scheduler::TaskPriority::Low,
        " ",
        Timestamp::now_utc(),
    );
    assert!(matches!(
        invalid_run,
        Err(agent_scheduler::SchedulerError::InvalidTask(_))
    ));

    let created = task_run("nightly index");
    let err = agent_scheduler::transition_task_state(
        &created,
        transition(TaskState::Created, TaskState::Queued, " "),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        agent_scheduler::SchedulerError::InvalidTransition(_)
    ));
}
