use ruse_scheduler::{
    InMemoryTaskQueue, QuotaDecision, QuotaPolicy, QuotaSnapshot, RetryPolicy, TaskKind,
    TaskPriority, TaskQueue, TaskRun, evaluate_task_quota,
};
use ruse_types::{ConversationId, Timestamp, WorkspaceId};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-06-25T00:00:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn task_run(
    kind: TaskKind,
    priority: TaskPriority,
    summary: &str,
    created_at: Timestamp,
) -> TaskRun {
    TaskRun::new(
        WorkspaceId::new(),
        Some(ConversationId::new()),
        kind,
        priority,
        summary,
        created_at,
    )
    .expect("valid task run")
}

fn policy() -> QuotaPolicy {
    QuotaPolicy {
        max_system_running: 4,
        max_workspace_running: 2,
        max_background_running: 2,
        max_cron_running: 1,
    }
}

fn snapshot() -> QuotaSnapshot {
    QuotaSnapshot {
        system_running: 1,
        workspace_running: 1,
        background_running: 1,
        cron_running: 0,
    }
}

fn rejection_code(decision: QuotaDecision) -> String {
    match decision {
        QuotaDecision::Allow => panic!("expected quota rejection"),
        QuotaDecision::Reject(rejection) => rejection.code,
    }
}

#[test]
fn quotas_allow_when_all_limits_have_capacity() {
    let candidate = task_run(TaskKind::Background, TaskPriority::Normal, "index", ts(0));

    let decision = evaluate_task_quota(&policy(), &snapshot(), &candidate);

    assert_eq!(decision, QuotaDecision::Allow);
}

#[test]
fn quotas_reject_system_limit() {
    let mut snapshot = snapshot();
    snapshot.system_running = 4;
    let candidate = task_run(TaskKind::OneShot, TaskPriority::Normal, "single", ts(0));

    let decision = evaluate_task_quota(&policy(), &snapshot, &candidate);

    assert_eq!(rejection_code(decision), "system_running_limit");
}

#[test]
fn quotas_reject_workspace_limit() {
    let mut snapshot = snapshot();
    snapshot.workspace_running = 2;
    let candidate = task_run(TaskKind::Background, TaskPriority::Normal, "index", ts(0));

    let decision = evaluate_task_quota(&policy(), &snapshot, &candidate);

    assert_eq!(rejection_code(decision), "workspace_running_limit");
}

#[test]
fn quotas_reject_background_and_cron_limits() {
    let mut background_snapshot = snapshot();
    background_snapshot.background_running = 2;
    let background = task_run(TaskKind::Background, TaskPriority::Normal, "index", ts(0));

    let mut cron_snapshot = snapshot();
    cron_snapshot.cron_running = 1;
    let cron = task_run(TaskKind::Cron, TaskPriority::Normal, "nightly", ts(0));

    let background_decision = evaluate_task_quota(&policy(), &background_snapshot, &background);
    let cron_decision = evaluate_task_quota(&policy(), &cron_snapshot, &cron);

    assert_eq!(
        rejection_code(background_decision),
        "background_running_limit"
    );
    assert_eq!(rejection_code(cron_decision), "cron_running_limit");
}

#[test]
fn quotas_policy_serializes_stable_shape() {
    let value = serde_json::to_value(policy()).expect("serialize quota policy");

    assert_eq!(value["max_system_running"], 4);
    assert_eq!(value["max_workspace_running"], 2);
    assert_eq!(value["max_background_running"], 2);
    assert_eq!(value["max_cron_running"], 1);
}

#[test]
fn quotas_queue_acquire_skips_rejected_candidate() {
    let mut queue = InMemoryTaskQueue::new(
        RetryPolicy {
            max_attempts: 3,
            base_delay_ms: 1_000,
            max_delay_ms: 5_000,
        },
        1_000,
    )
    .expect("valid queue");
    let rejected_background = task_run(TaskKind::Background, TaskPriority::High, "index", ts(0));
    let allowed_cron = task_run(TaskKind::Cron, TaskPriority::Low, "nightly", ts(1));
    let rejected_id = rejected_background.task_id;
    let allowed_id = allowed_cron.task_id;
    let quota_policy = QuotaPolicy {
        max_system_running: 4,
        max_workspace_running: 4,
        max_background_running: 0,
        max_cron_running: 1,
    };
    let quota_snapshot = QuotaSnapshot {
        system_running: 0,
        workspace_running: 0,
        background_running: 0,
        cron_running: 0,
    };
    queue.enqueue(rejected_background).unwrap();
    queue.enqueue(allowed_cron).unwrap();

    let leased = queue
        .acquire_next_with_quota("worker-a", ts(2), &quota_policy, &quota_snapshot)
        .unwrap()
        .expect("quota-aware acquisition skips rejected task");
    let still_queued = queue
        .acquire_next("worker-b", ts(3))
        .unwrap()
        .expect("rejected task remains queued");

    assert_eq!(leased.task_id, allowed_id);
    assert_eq!(still_queued.task_id, rejected_id);
}
