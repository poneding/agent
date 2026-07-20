use ruse_core::{
    BackgroundTaskRunner, BackgroundTaskRunnerInput, BackgroundTaskTurnError,
    BackgroundTaskTurnExecutor, BackgroundTaskTurnResult,
};
use ruse_scheduler::{
    InMemoryTaskNotificationSink, InMemoryTaskQueue, RetryPolicy, TaskKind, TaskNotificationKind,
    TaskPriority, TaskQueue, TaskRun,
};
use ruse_types::{ConversationId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-29T01:00:{second:02}Z\"")).expect("valid timestamp")
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
async fn background_task_cancellation_cancels_lease_before_execution_and_notifies() {
    let mut queue = queue();
    let task = task_run("cancel me");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let mut notifications = InMemoryTaskNotificationSink::default();
    let mut executor = CountingExecutor::default();

    let outcome = BackgroundTaskRunner
        .run_one_with_notifications(
            &mut queue,
            &mut executor,
            &mut notifications,
            BackgroundTaskRunnerInput::new("worker-a", ts(1))
                .expect("input")
                .with_cancellation_requested(true),
        )
        .await
        .expect("runner")
        .expect("outcome");

    assert_eq!(outcome.task_id, task_id);
    assert_eq!(outcome.final_state, TaskState::Cancelled);
    assert_eq!(executor.executions, 0);
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Cancelled
    );
    assert!(notifications.notifications().iter().any(|notification| {
        notification.task_id == task_id && notification.kind == TaskNotificationKind::Cancelled
    }));
}

#[derive(Default)]
struct CountingExecutor {
    executions: usize,
}

#[async_trait::async_trait]
impl BackgroundTaskTurnExecutor for CountingExecutor {
    async fn execute_task_turn(
        &mut self,
        _task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError> {
        self.executions += 1;
        Ok(BackgroundTaskTurnResult::completed("done", 1, None))
    }
}
