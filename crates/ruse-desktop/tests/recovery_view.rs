use ruse_desktop::{DesktopTaskItem, build_task_monitor};
use ruse_types::{
    ConversationId, RecoveryConflictKind, RecoveryConflictNextAction, RecoveryConflictSide,
    RecoveryConflictSummary, RecoveryDiagnostic, RecoveryDiagnosticSeverity, TaskId, Timestamp,
    WorkspaceId,
};
use ruse_ui_bridge::{UiTaskScheduleSummary, UiTaskSnapshot, UiTaskSummary};

fn ts(second: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-02T00:01:{second:02}Z\""))
        .expect("valid fixed timestamp")
}

fn workspace_id() -> WorkspaceId {
    "00000000-0000-0000-0000-000000006204"
        .parse()
        .expect("workspace id")
}

fn conversation_id() -> ConversationId {
    "00000000-0000-0000-0000-000000006205"
        .parse()
        .expect("conversation id")
}

fn task_id() -> TaskId {
    "00000000-0000-0000-0000-000000006206"
        .parse()
        .expect("task id")
}

fn recovery_conflict() -> RecoveryConflictSummary {
    RecoveryConflictSummary {
        path: "src/[redacted].rs".to_string(),
        kind: RecoveryConflictKind::BothModified,
        lead_change: RecoveryConflictSide {
            source: "workspace".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("[redacted]".to_string()),
        },
        incoming_change: RecoveryConflictSide {
            source: "recovery".to_string(),
            change_kind: "modified".to_string(),
            content_hash: Some("incoming-safe-hash".to_string()),
        },
        next_action: RecoveryConflictNextAction::InspectDiff,
        diagnostics: vec![RecoveryDiagnostic {
            severity: RecoveryDiagnosticSeverity::Warning,
            code: "recovery.conflict.blocks_resume".to_string(),
            message: "manual review required before resume".to_string(),
            source: Some("fixture".to_string()),
            truncated: false,
        }],
    }
}

fn task_with_conflict() -> UiTaskSummary {
    let mut task: UiTaskSummary = serde_json::from_value(serde_json::json!({
        "task_id": task_id(),
        "workspace_id": workspace_id(),
        "conversation_id": conversation_id(),
        "state": "failed",
        "priority": "normal",
        "schedule": UiTaskScheduleSummary {
            kind: "one_shot".to_string(),
            label: "run once".to_string(),
        },
        "prompt_summary": "Recover conflicted turn",
        "latest_run_summary": "blocked by recovery conflict",
        "created_at": ts(1),
        "updated_at": ts(2),
    }))
    .expect("ui task summary");
    task.recovery_conflicts = vec![recovery_conflict()];
    task
}

#[test]
fn desktop_task_monitor_preserves_recovery_conflict_summaries() {
    let snapshot = UiTaskSnapshot::new(vec![task_with_conflict()], Vec::new()).expect("snapshot");

    let monitor = build_task_monitor(&snapshot).expect("monitor");
    let item = &monitor.tasks[0];

    assert_eq!(item.recovery_conflicts.len(), 1);
    assert_eq!(item.recovery_conflicts[0].path, "src/[redacted].rs");
    assert_eq!(
        item.recovery_conflicts[0].kind,
        RecoveryConflictKind::BothModified
    );
    assert_eq!(
        item.recovery_conflicts[0].next_action,
        RecoveryConflictNextAction::InspectDiff
    );

    let serialized = serde_json::to_string(item).expect("serialize monitor item");
    assert!(serialized.contains("recovery_conflicts"));
    assert!(serialized.contains("src/[redacted].rs"));
    assert!(!serialized.contains("token-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("sk-secret"));
}

#[test]
fn desktop_task_item_preserves_recovery_conflicts() {
    let task = task_with_conflict();
    let item = DesktopTaskItem::try_from(&task).expect("desktop task");

    assert_eq!(item.recovery_conflicts.len(), 1);
    assert_eq!(
        item.recovery_conflicts[0]
            .lead_change
            .content_hash
            .as_deref(),
        Some("[redacted]")
    );
}
