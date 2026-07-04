use agent_core::{TaskArtifact, TaskArtifactKind, TaskReviewSummary};
use agent_types::{CheckpointId, TaskId, TaskState};
use agent_ui_bridge::{
    UiJsonFrame, UiTaskReviewSnapshot, UiTaskReviewSummary, UiTaskReviewSummaryArtifact,
};

fn task_id(value: &str) -> TaskId {
    value.parse().expect("task id")
}

fn checkpoint_id() -> CheckpointId {
    "00000000-0000-0000-0000-000000002425"
        .parse()
        .expect("checkpoint id")
}

#[test]
fn task_review_snapshot_serializes_safe_review_and_artifacts() {
    let task_id = task_id("00000000-0000-0000-0000-000000002401");
    let artifact = TaskArtifact::new(
        "artifact-report",
        task_id,
        TaskArtifactKind::Report,
        Some("artifacts/release-report.md"),
        Some("sha256:abc123"),
        "release summary with api_key secret",
    )
    .expect("artifact");
    let review = TaskReviewSummary::from_artifacts(
        task_id,
        TaskState::Succeeded,
        Some("trace-123"),
        Some(checkpoint_id()),
        std::slice::from_ref(&artifact),
        "finished background review with sk-secret-token",
    )
    .expect("review");

    let ui_review = UiTaskReviewSummary::from_review(&review, std::slice::from_ref(&artifact))
        .expect("ui review");
    let snapshot = UiTaskReviewSnapshot::new(vec![ui_review], Vec::new()).expect("snapshot");
    let serialized =
        serde_json::to_string(&UiJsonFrame::TaskReviewSnapshot(snapshot)).expect("serialize");

    assert!(serialized.contains("\"type\":\"task_review_snapshot\""));
    assert!(serialized.contains("\"final_state\":\"succeeded\""));
    assert!(serialized.contains("\"artifact_id\":\"artifact-report\""));
    assert!(serialized.contains("\"checkpoint_id\""));
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret-token"));
    assert!(!serialized.contains("api_key"));
}

#[test]
fn task_review_snapshot_reports_missing_artifact_metadata() {
    let task_id = task_id("00000000-0000-0000-0000-000000002402");
    let review = TaskReviewSummary::new(
        task_id,
        TaskState::Failed,
        Some("trace-456"),
        None,
        vec!["artifact-token-missing".to_string()],
        "failed after tool error with sk-secret-token",
    )
    .expect("review");

    let ui_review = UiTaskReviewSummary::from_review(&review, &[]).expect("ui review");

    assert_eq!(
        ui_review.artifacts,
        Vec::<UiTaskReviewSummaryArtifact>::new()
    );
    assert_eq!(
        ui_review.diagnostics,
        vec!["missing artifact metadata: [redacted]".to_string()]
    );
    assert!(ui_review.summary.contains("[redacted]"));
}
