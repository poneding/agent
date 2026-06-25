use agent_scheduler::{
    InMemoryTaskQueue, LeaseStatus, RetryDecision, RetryPolicy, TaskKind, TaskPriority, TaskQueue,
    TaskRun,
};
use agent_types::{ConversationId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn task_run(summary: &str, priority: TaskPriority, created_at: Timestamp) -> TaskRun {
    TaskRun::new(
        WorkspaceId::new(),
        Some(ConversationId::new()),
        TaskKind::Background,
        priority,
        summary,
        created_at,
    )
    .expect("valid task run")
}

fn queue() -> InMemoryTaskQueue {
    InMemoryTaskQueue::new(
        RetryPolicy {
            max_attempts: 3,
            base_delay_ms: 1_000,
            max_delay_ms: 5_000,
        },
        1_000,
    )
    .expect("valid queue")
}

#[test]
fn queue_leases_acquires_queued_task_and_creates_active_lease() {
    let mut queue = queue();
    let older_normal = task_run("normal older", TaskPriority::Normal, ts(0));
    let newer_normal = task_run("normal newer", TaskPriority::Normal, ts(1));
    let newer_high = task_run("high newer", TaskPriority::High, ts(2));
    let high_id = newer_high.task_id;
    let older_normal_id = older_normal.task_id;

    queue.enqueue(older_normal).unwrap();
    queue.enqueue(newer_normal).unwrap();
    queue.enqueue(newer_high).unwrap();

    let first = queue
        .acquire_next("worker-a", ts(3))
        .unwrap()
        .expect("high-priority task is leased first");
    let second = queue
        .acquire_next("worker-a", ts(4))
        .unwrap()
        .expect("oldest normal-priority task is leased second");

    assert_eq!(first.task_id, high_id);
    assert_eq!(first.worker_id, "worker-a");
    assert_eq!(first.acquired_at, ts(3));
    assert_eq!(first.expires_at, ts(4));
    assert_eq!(first.status, LeaseStatus::Active);
    assert_eq!(second.task_id, older_normal_id);
    assert_eq!(second.status, LeaseStatus::Active);
}

#[test]
fn queue_leases_complete_requires_active_lease() {
    let mut queue = queue();
    queue
        .enqueue(task_run("complete once", TaskPriority::Normal, ts(0)))
        .unwrap();
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .unwrap()
        .expect("task is leased");

    let completed = queue.complete(&lease.id, ts(2)).unwrap();
    let duplicate = queue.complete(&lease.id, ts(3)).unwrap_err();
    let terminal_cancel = queue
        .cancel(completed.task_id, "too late".to_string(), ts(4))
        .unwrap_err();

    assert_eq!(completed.state, TaskState::Succeeded);
    assert!(matches!(
        duplicate,
        agent_scheduler::SchedulerError::InvalidLease(_)
    ));
    assert!(matches!(
        terminal_cancel,
        agent_scheduler::SchedulerError::InvalidTransition(_)
    ));
}

#[test]
fn queue_leases_failure_schedules_retry_until_exhausted() {
    let mut queue = queue();
    let mut run = task_run("retry twice", TaskPriority::Normal, ts(0));
    run.max_attempts = 2;
    let task_id = run.task_id;
    queue.enqueue(run).unwrap();

    let first_lease = queue
        .acquire_next("worker-a", ts(1))
        .unwrap()
        .expect("task is leased");
    let retry = queue
        .fail(&first_lease.id, "tool failed".to_string(), ts(2))
        .unwrap();

    match retry {
        RetryDecision::RetryAt { task, run_at } => {
            assert_eq!(task.task_id, task_id);
            assert_eq!(task.state, TaskState::Sleeping);
            assert_eq!(task.attempt, 1);
            assert_eq!(run_at, ts(3));
        }
        RetryDecision::Exhausted { .. } => panic!("first failure should retry"),
    }

    assert!(queue.acquire_next("worker-a", ts(2)).unwrap().is_none());
    let retry_lease = queue
        .acquire_next("worker-a", ts(3))
        .unwrap()
        .expect("sleeping task wakes when retry time arrives");
    assert_eq!(retry_lease.task_id, task_id);

    let exhausted = queue
        .fail(&retry_lease.id, "still broken".to_string(), ts(4))
        .unwrap();
    match exhausted {
        RetryDecision::RetryAt { .. } => panic!("second failure should exhaust"),
        RetryDecision::Exhausted { task, error } => {
            assert_eq!(task.task_id, task_id);
            assert_eq!(task.state, TaskState::Failed);
            assert_eq!(task.attempt, 2);
            assert_eq!(error, "still broken");
        }
    }
}

#[test]
fn queue_leases_cancel_pause_and_resume_update_state() {
    let mut queue = queue();
    let queued = task_run("cancel queued", TaskPriority::Normal, ts(0));
    let queued_id = queued.task_id;
    queue.enqueue(queued).unwrap();
    let cancelled = queue
        .cancel(queued_id, "user cancelled".to_string(), ts(1))
        .unwrap();
    assert_eq!(cancelled.state, TaskState::Cancelled);

    let running = task_run("pause running", TaskPriority::Normal, ts(2));
    let running_id = running.task_id;
    queue.enqueue(running).unwrap();
    let lease = queue
        .acquire_next("worker-a", ts(3))
        .unwrap()
        .expect("task is leased");

    let paused = queue
        .pause(running_id, "wait for approval".to_string(), ts(4))
        .unwrap();
    let stale_complete = queue.complete(&lease.id, ts(5)).unwrap_err();
    let resumed = queue
        .resume(running_id, "approved".to_string(), ts(6))
        .unwrap();
    let next_lease = queue
        .acquire_next("worker-a", ts(7))
        .unwrap()
        .expect("resumed task can be leased again");

    assert_eq!(paused.state, TaskState::Paused);
    assert!(matches!(
        stale_complete,
        agent_scheduler::SchedulerError::InvalidLease(_)
    ));
    assert_eq!(resumed.state, TaskState::Queued);
    assert_eq!(next_lease.task_id, running_id);
}

#[test]
fn queue_leases_release_expired_leases_requeues_task() {
    let mut queue = queue();
    let run = task_run("lease expires", TaskPriority::Normal, ts(0));
    let task_id = run.task_id;
    queue.enqueue(run).unwrap();
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .unwrap()
        .expect("task is leased");

    assert!(queue.release_expired_leases(ts(1)).is_empty());
    let expired = queue.release_expired_leases(ts(2));
    let stale_complete = queue.complete(&lease.id, ts(3)).unwrap_err();
    let next_lease = queue
        .acquire_next("worker-b", ts(3))
        .unwrap()
        .expect("expired lease returns task to queue");

    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].id, lease.id);
    assert_eq!(expired[0].status, LeaseStatus::Expired);
    assert!(matches!(
        stale_complete,
        agent_scheduler::SchedulerError::InvalidLease(_)
    ));
    assert_eq!(next_lease.task_id, task_id);
    assert_eq!(next_lease.worker_id, "worker-b");
}
