use agent_core::{
    BackgroundApprovalDecision, BackgroundApprovalDecisionKind, BackgroundTaskRunner,
    BackgroundTaskRunnerInput, BackgroundTaskTurnError, BackgroundTaskTurnExecutor,
    BackgroundTaskTurnResult,
};
use agent_scheduler::{
    InMemoryTaskNotificationSink, InMemoryTaskQueue, RetryPolicy, TaskKind, TaskNotificationKind,
    TaskPriority, TaskQueue, TaskRun,
};
use agent_types::{ApprovalRequestId, ConversationId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-28T00:00:{second:02}Z\"")).expect("valid timestamp")
}

fn approval_id() -> ApprovalRequestId {
    "00000000-0000-0000-0000-000000002403"
        .parse()
        .expect("approval id")
}

fn task_run(summary: &str) -> TaskRun {
    TaskRun::new(
        WorkspaceId::new(),
        Some(ConversationId::new()),
        TaskKind::Background,
        TaskPriority::Normal,
        summary,
        ts(0),
    )
    .expect("task")
}

fn queue() -> InMemoryTaskQueue {
    InMemoryTaskQueue::new(RetryPolicy::default(), 1_000).expect("queue")
}

#[tokio::test]
async fn background_task_approvals_waiting_releases_lease_and_notifies() {
    let mut queue = queue();
    let task = task_run("requires approval");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let mut notifications = InMemoryTaskNotificationSink::default();
    let mut executor = ApprovalExecutor::waiting("approve shell command with sk-secret-token");

    let outcome = BackgroundTaskRunner
        .run_one_with_notifications(
            &mut queue,
            &mut executor,
            &mut notifications,
            BackgroundTaskRunnerInput::new("worker-a", ts(1)).expect("input"),
        )
        .await
        .expect("runner")
        .expect("outcome");

    assert_eq!(outcome.task_id, task_id);
    assert_eq!(outcome.final_state, TaskState::WaitingApproval);
    assert_eq!(outcome.approval_id, Some(approval_id()));
    assert!(outcome.summary.contains("[redacted]"));
    assert!(!outcome.summary.contains("sk-secret-token"));
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::WaitingApproval
    );
    let task = queue.get_task(task_id).expect("task");
    assert_eq!(task.waiting_approval_id, Some(approval_id()));
    let latest = task
        .latest_notification
        .as_ref()
        .expect("latest notification");
    assert_eq!(latest.kind, TaskNotificationKind::WaitingApproval);
    assert_eq!(latest.approval_id, Some(approval_id()));
    assert!(latest.summary.contains("[redacted]"));
    assert!(!latest.summary.contains("sk-secret-token"));
    assert!(notifications.notifications().iter().any(|notification| {
        notification.task_id == task_id
            && notification.kind == TaskNotificationKind::WaitingApproval
            && notification.approval_id == Some(approval_id())
    }));
}

#[test]
fn background_task_approvals_allow_resumes_queued_continuation() {
    let mut queue = queue();
    let task = task_run("approval resume");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .expect("lease")
        .expect("lease");
    queue
        .wait_for_approval(&lease.id, approval_id(), "waiting", ts(2))
        .expect("waiting");
    let mut notifications = InMemoryTaskNotificationSink::default();

    let outcome = BackgroundTaskRunner
        .resolve_approval(
            &mut queue,
            &mut notifications,
            BackgroundApprovalDecision::new(
                task_id,
                approval_id(),
                BackgroundApprovalDecisionKind::Allow,
                "approved continuation",
                ts(3),
            )
            .expect("decision"),
        )
        .expect("resolve approval");

    assert_eq!(outcome.final_state, TaskState::Queued);
    assert_eq!(outcome.approval_id, Some(approval_id()));
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Queued
    );
    let task = queue.get_task(task_id).expect("task");
    assert_eq!(task.waiting_approval_id, None);
    assert_eq!(
        task.latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Resumed)
    );
    assert!(notifications.notifications().iter().any(|notification| {
        notification.kind == TaskNotificationKind::Resumed
            && notification.approval_id == Some(approval_id())
    }));
}

#[test]
fn background_task_approvals_deny_cancels_without_execution() {
    let mut queue = queue();
    let task = task_run("approval deny");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .expect("lease")
        .expect("lease");
    queue
        .wait_for_approval(&lease.id, approval_id(), "waiting", ts(2))
        .expect("waiting");
    let mut notifications = InMemoryTaskNotificationSink::default();

    let outcome = BackgroundTaskRunner
        .resolve_approval(
            &mut queue,
            &mut notifications,
            BackgroundApprovalDecision::new(
                task_id,
                approval_id(),
                BackgroundApprovalDecisionKind::Deny,
                "too risky",
                ts(3),
            )
            .expect("decision"),
        )
        .expect("resolve approval");

    assert_eq!(outcome.final_state, TaskState::Cancelled);
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Cancelled
    );
    let task = queue.get_task(task_id).expect("task");
    assert_eq!(task.waiting_approval_id, None);
    assert_eq!(
        task.latest_notification
            .as_ref()
            .map(|notification| notification.kind),
        Some(TaskNotificationKind::Cancelled)
    );
    assert!(notifications.notifications().iter().any(|notification| {
        notification.kind == TaskNotificationKind::Cancelled
            && notification.approval_id == Some(approval_id())
    }));
}

struct ApprovalExecutor {
    result: BackgroundTaskTurnResult,
}

impl ApprovalExecutor {
    fn waiting(summary: &str) -> Self {
        Self {
            result: BackgroundTaskTurnResult::waiting_approval(approval_id(), summary, 2),
        }
    }
}

#[async_trait::async_trait]
impl BackgroundTaskTurnExecutor for ApprovalExecutor {
    async fn execute_task_turn(
        &mut self,
        _task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError> {
        Ok(self.result.clone())
    }
}
