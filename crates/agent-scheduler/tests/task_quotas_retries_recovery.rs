use agent_scheduler::{
    InMemoryTaskQueue, RetryDecision, RetryPolicy, StaleLeaseDisposition, StaleLeaseRecoveryPolicy,
    TaskKind, TaskPriority, TaskQueue, TaskRun, TaskUsageQuotaPolicy, TaskUsageSnapshot,
    evaluate_task_usage_quota,
};
use agent_types::{ConversationId, TaskState, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-29T00:00:{second:02}Z\"")).expect("valid timestamp")
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
    InMemoryTaskQueue::new(
        RetryPolicy {
            max_attempts: 3,
            base_delay_ms: 1_000,
            max_delay_ms: 4_000,
        },
        1_000,
    )
    .expect("queue")
}

#[test]
fn task_quotas_reject_usage_dimensions_with_stable_codes() {
    let policy = TaskUsageQuotaPolicy {
        max_model_calls: 2,
        max_tool_calls: 3,
        max_output_bytes: 100,
        max_elapsed_ms: 1_000,
        max_worktree_count: 1,
    };

    let cases = [
        (
            TaskUsageSnapshot {
                model_calls: 2,
                ..TaskUsageSnapshot::default()
            },
            "model_call_limit",
        ),
        (
            TaskUsageSnapshot {
                tool_calls: 3,
                ..TaskUsageSnapshot::default()
            },
            "tool_call_limit",
        ),
        (
            TaskUsageSnapshot {
                output_bytes: 100,
                ..TaskUsageSnapshot::default()
            },
            "output_byte_limit",
        ),
        (
            TaskUsageSnapshot {
                elapsed_ms: 1_000,
                ..TaskUsageSnapshot::default()
            },
            "wall_time_limit",
        ),
        (
            TaskUsageSnapshot {
                worktree_count: 1,
                ..TaskUsageSnapshot::default()
            },
            "worktree_limit",
        ),
    ];

    for (usage, expected_code) in cases {
        let decision = evaluate_task_usage_quota(&policy, &usage);
        let rejection = decision.into_rejection().expect("usage should be rejected");
        assert_eq!(rejection.code, expected_code);
        assert!(!rejection.message.contains("sk-secret-token"));
    }
}

#[test]
fn task_quotas_allow_usage_below_limits() {
    let policy = TaskUsageQuotaPolicy {
        max_model_calls: 2,
        max_tool_calls: 3,
        max_output_bytes: 100,
        max_elapsed_ms: 1_000,
        max_worktree_count: 1,
    };
    let usage = TaskUsageSnapshot {
        model_calls: 1,
        tool_calls: 2,
        output_bytes: 99,
        elapsed_ms: 999,
        worktree_count: 0,
    };

    assert!(evaluate_task_usage_quota(&policy, &usage).is_allow());
}

#[test]
fn task_retries_expose_visible_attempt_summary() {
    let mut queue = queue();
    let mut run = task_run("retry summary");
    run.max_attempts = 2;
    let task_id = run.task_id;
    queue.enqueue(run).expect("enqueue");
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .expect("lease")
        .expect("lease");

    let retry = queue
        .fail(
            &lease.id,
            "provider timeout with sk-secret-token".to_string(),
            ts(2),
        )
        .expect("retry");
    let summary = retry.summary();

    assert_eq!(summary.task_id, task_id);
    assert_eq!(summary.attempt, 1);
    assert_eq!(summary.max_attempts, 2);
    assert_eq!(summary.next_run_at, Some(ts(3)));
    assert_eq!(
        summary.error.as_deref(),
        Some("provider timeout with [redacted]")
    );

    let RetryDecision::RetryAt { task, .. } = retry else {
        panic!("first failure should retry");
    };
    let retry_lease = queue
        .acquire_next("worker-a", ts(3))
        .expect("retry lease")
        .expect("retry lease");
    assert_eq!(retry_lease.task_id, task.task_id);

    let exhausted = queue
        .fail(&retry_lease.id, "still broken".to_string(), ts(4))
        .expect("exhausted");
    let exhausted_summary = exhausted.summary();

    assert_eq!(exhausted_summary.task_id, task_id);
    assert_eq!(exhausted_summary.state, TaskState::Failed);
    assert_eq!(exhausted_summary.next_run_at, None);
    assert_eq!(exhausted_summary.error.as_deref(), Some("still broken"));
}

#[test]
fn stale_lease_recovery_reports_requeued_tasks() {
    let mut queue = queue();
    let task = task_run("requeue stale lease");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .expect("lease")
        .expect("lease");

    let report = queue.recover_expired_leases(&StaleLeaseRecoveryPolicy::requeue(), ts(2));

    assert_eq!(report.expired_leases, 1);
    assert_eq!(report.requeued_tasks, 1);
    assert_eq!(report.failed_tasks, 0);
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Queued
    );
    assert!(queue.complete(&lease.id, ts(3)).is_err());
}

#[test]
fn stale_lease_recovery_can_fail_expired_tasks() {
    let mut queue = queue();
    let task = task_run("fail stale lease");
    let task_id = task.task_id;
    queue.enqueue(task).expect("enqueue");
    let lease = queue
        .acquire_next("worker-a", ts(1))
        .expect("lease")
        .expect("lease");

    let report = queue.recover_expired_leases(
        &StaleLeaseRecoveryPolicy {
            disposition: StaleLeaseDisposition::Fail,
        },
        ts(2),
    );

    assert_eq!(report.expired_leases, 1);
    assert_eq!(report.requeued_tasks, 0);
    assert_eq!(report.failed_tasks, 1);
    assert_eq!(
        queue.get_task(task_id).expect("task").state,
        TaskState::Failed
    );
    assert!(queue.complete(&lease.id, ts(3)).is_err());
}
