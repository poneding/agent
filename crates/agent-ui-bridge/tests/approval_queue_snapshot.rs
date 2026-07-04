use agent_types::Timestamp;
use agent_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalQueueSnapshot, UiApprovalRequest, UiEventEnvelope,
    UiEventPayload, UiJsonFrame, approval_queue_snapshot_from_events,
    approval_queue_snapshot_from_requests, encode_json_frame,
};

fn ts(day: u8) -> Timestamp {
    serde_json::from_str(&format!("\"2026-07-{day:02}T00:00:00Z\"")).expect("timestamp")
}

fn approval_request(
    id: &str,
    resource_kind: &str,
    action: &str,
    resource: &str,
) -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: id.to_string(),
        status: "pending".to_string(),
        action: action.to_string(),
        resource_kind: resource_kind.to_string(),
        resource: resource.to_string(),
        reason: "verify before running".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: resource_kind.to_string(),
            resource: resource.to_string(),
            summary: format!("Review {resource}"),
            content_hash: Some("sha256:redacted".to_string()),
            truncated: false,
            redacted: true,
        },
        max_duration_seconds: Some(300),
        expires_at: Some(ts(3)),
    }
}

fn approval_event(payload: UiEventPayload) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: agent_types::SchemaVersion::current(),
        event_id: agent_types::EventId::new(),
        seq: None,
        occurred_at: ts(1),
        conversation_id: agent_types::ConversationId::new(),
        turn_id: None,
        task_id: None,
        correlation_id: agent_types::CorrelationId::new(),
        payload,
    })
}

#[test]
fn approval_queue_empty_snapshot_is_successful() {
    let snapshot = approval_queue_snapshot_from_requests(Vec::new()).expect("snapshot");

    assert!(snapshot.pending.is_empty());
    assert!(snapshot.groups.is_empty());
    assert_eq!(snapshot.pending_count, 0);
    assert_eq!(snapshot.expired_count, 0);
    assert!(snapshot.diagnostics.is_empty());
}

#[test]
fn approval_queue_sorts_collapses_duplicates_and_groups_pending_items() {
    let old = approval_request(
        "00000000-0000-0000-0000-000000000001",
        "command",
        "execute",
        "cargo test",
    );
    let mut new = old.clone();
    new.reason = "latest reason".to_string();
    new.resource = "cargo clippy".to_string();
    let read = approval_request(
        "00000000-0000-0000-0000-000000000002",
        "file",
        "read",
        "src/lib.rs",
    );

    let snapshot = approval_queue_snapshot_from_requests(vec![read, old, new]).expect("snapshot");

    assert_eq!(snapshot.pending_count, 2);
    assert_eq!(snapshot.pending.len(), 2);
    assert_eq!(snapshot.pending[0].resource_kind, "command");
    assert_eq!(snapshot.pending[0].resource, "cargo clippy");
    assert_eq!(snapshot.pending[0].reason, "latest reason");
    assert_eq!(snapshot.groups.len(), 2);
    assert!(snapshot.groups.iter().any(|group| {
        group.resource_kind == "command" && group.action == "execute" && group.count == 1
    }));
}

#[test]
fn approval_queue_events_remove_resolved_requests() {
    let request = approval_request(
        "00000000-0000-0000-0000-000000000001",
        "command",
        "execute",
        "cargo test",
    );
    let frames = vec![
        approval_event(UiEventPayload::ApprovalRequested(Box::new(request))),
        approval_event(UiEventPayload::ApprovalResolved {
            request_id: "00000000-0000-0000-0000-000000000001".to_string(),
            status: "approved".to_string(),
            decision: Some("allow".to_string()),
            reason: Some("approved".to_string()),
        }),
    ];

    let snapshot = approval_queue_snapshot_from_events(&frames).expect("snapshot");

    assert!(snapshot.pending.is_empty());
    assert_eq!(snapshot.pending_count, 0);
}

#[test]
fn approval_queue_redacts_and_bounds_public_text() {
    let request = approval_request(
        "00000000-0000-0000-0000-000000000001",
        "command-secret",
        "execute-token",
        "sk-test-secret ".repeat(20).trim(),
    );

    let snapshot = approval_queue_snapshot_from_requests(vec![request]).expect("snapshot");
    let rendered = serde_json::to_string(&snapshot).expect("serialize");

    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("command-secret"));
    assert!(!rendered.contains("execute-token"));
    assert!(snapshot.pending[0].resource.chars().count() <= 160);
}

#[test]
fn approval_queue_snapshot_json_frame_is_single_line() {
    let snapshot = UiApprovalQueueSnapshot::default();

    let line = encode_json_frame(&UiJsonFrame::ApprovalQueueSnapshot(snapshot)).expect("encode");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("frame");
    assert!(matches!(frame, UiJsonFrame::ApprovalQueueSnapshot(_)));
}
