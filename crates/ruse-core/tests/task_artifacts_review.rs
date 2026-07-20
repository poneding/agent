use ruse_core::{
    BackgroundTaskRunner, BackgroundTaskRunnerInput, BackgroundTaskTurnError,
    BackgroundTaskTurnExecutor, BackgroundTaskTurnResult, TaskArtifact, TaskArtifactKind,
    TaskReviewSummary,
};
use ruse_scheduler::{InMemoryTaskQueue, RetryPolicy, TaskKind, TaskPriority, TaskQueue, TaskRun};
use ruse_types::{CheckpointId, ConversationId, TaskId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-29T02:00:{second:02}Z\"")).expect("valid timestamp")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000002405"
        .parse()
        .expect("task id")
}

fn checkpoint_id() -> CheckpointId {
    "00000000-0000-0000-0000-000000002415"
        .parse()
        .expect("checkpoint id")
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

#[test]
fn task_artifacts_review_serializes_safe_metadata() {
    let artifact = TaskArtifact::new(
        "artifact-report",
        task_id(),
        TaskArtifactKind::Report,
        Some("artifacts/release-report.md"),
        Some("sha256:abc123"),
        "created release report with sk-secret-token",
    )
    .expect("artifact");

    assert_eq!(artifact.summary, "created release report with [redacted]");
    assert_eq!(
        artifact.path.as_deref(),
        Some("artifacts/release-report.md")
    );

    let value = serde_json::to_value(&artifact).expect("serialize artifact");
    assert_eq!(value["artifact_id"], "artifact-report");
    assert_eq!(value["kind"], "report");
    assert_eq!(value["content_hash"], "sha256:abc123");
    assert_eq!(value["summary"], "created release report with [redacted]");
}

#[test]
fn task_artifacts_review_records_missing_artifact_diagnostics() {
    let artifact = TaskArtifact::missing(
        "artifact-trace",
        task_id(),
        TaskArtifactKind::Trace,
        "trace artifact unavailable",
        "artifact file was missing at review time",
    )
    .expect("missing artifact");

    assert_eq!(artifact.path, None);
    assert!(artifact.missing);
    assert_eq!(
        artifact.diagnostics,
        vec!["artifact file was missing at review time".to_string()]
    );
}

#[test]
fn task_review_summary_bounds_and_redacts_completion_context() {
    let artifact = TaskArtifact::new(
        "artifact-report",
        task_id(),
        TaskArtifactKind::Report,
        Some("artifacts/release-report.md"),
        None,
        "safe artifact",
    )
    .expect("artifact");
    let review = TaskReviewSummary::from_artifacts(
        task_id(),
        TaskState::Succeeded,
        Some("trace-123"),
        Some(checkpoint_id()),
        &[artifact],
        "completed task with password secret",
    )
    .expect("review");

    assert_eq!(review.final_state, TaskState::Succeeded);
    assert_eq!(review.trace_id.as_deref(), Some("trace-123"));
    assert_eq!(review.checkpoint_id, Some(checkpoint_id()));
    assert_eq!(review.artifact_ids, vec!["artifact-report".to_string()]);
    assert_eq!(review.summary, "completed task with [redacted] [redacted]");
}

#[tokio::test]
async fn background_runner_returns_artifacts_and_review_summary() {
    let mut queue = queue();
    let task = task_run("write a review artifact");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let mut executor = ArtifactExecutor;

    let outcome = BackgroundTaskRunner
        .run_one(
            &mut queue,
            &mut executor,
            BackgroundTaskRunnerInput::new("worker-a", ts(1)).expect("input"),
        )
        .await
        .expect("runner")
        .expect("outcome");

    assert_eq!(outcome.task_id, task_id);
    assert_eq!(outcome.final_state, TaskState::Succeeded);
    assert_eq!(outcome.review.final_state, TaskState::Succeeded);
    assert_eq!(outcome.review.artifact_ids, vec!["artifact-report"]);
    assert_eq!(outcome.review.checkpoint_id, Some(checkpoint_id()));
    assert_eq!(outcome.artifacts.len(), 1);
    assert_eq!(outcome.artifacts[0].task_id, task_id);
    assert!(!outcome.review.summary.contains("sk-secret-token"));
}

struct ArtifactExecutor;

#[async_trait::async_trait]
impl BackgroundTaskTurnExecutor for ArtifactExecutor {
    async fn execute_task_turn(
        &mut self,
        task: &TaskRun,
    ) -> Result<BackgroundTaskTurnResult, BackgroundTaskTurnError> {
        let artifact = TaskArtifact::new(
            "artifact-report",
            task.task_id,
            TaskArtifactKind::Report,
            Some("artifacts/release-report.md"),
            Some("sha256:abc123"),
            "release report",
        )
        .expect("artifact");

        Ok(BackgroundTaskTurnResult::completed(
            "completed with sk-secret-token",
            3,
            Some("trace-123".to_string()),
        )
        .with_checkpoint_id(checkpoint_id())
        .with_artifacts(vec![artifact]))
    }
}
