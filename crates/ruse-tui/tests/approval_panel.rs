use ruse_tui::{
    ApprovalPanelAction, ApprovalPanelState, TuiApp, TuiBridgeContext, TuiEffect,
    render_app_to_string,
};
use ruse_types::{ClientKind, ConversationId, WorkspaceId};
use ruse_ui_bridge::{
    UiApprovalDecision, UiApprovalPreviewSummary, UiApprovalQueueSnapshot, UiApprovalRequest,
    UiCommand, UiEventEnvelope, UiEventPayload, UiJsonFrame, approval_queue_snapshot_from_requests,
};

fn approval_request(id: &str, resource: &str) -> UiApprovalRequest {
    UiApprovalRequest {
        request_id: id.to_string(),
        status: "pending".to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: resource.to_string(),
        reason: "run verification".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: resource.to_string(),
            summary: format!("Run {resource}"),
            content_hash: Some("sha256:redacted".to_string()),
            truncated: false,
            redacted: true,
        },
        max_duration_seconds: Some(300),
        expires_at: None,
    }
}

fn approval_frame(request: UiApprovalRequest) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: ruse_types::SchemaVersion::current(),
        event_id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: ruse_types::Timestamp::now_utc(),
        conversation_id: ruse_types::ConversationId::new(),
        turn_id: None,
        task_id: None,
        correlation_id: ruse_types::CorrelationId::new(),
        payload: UiEventPayload::ApprovalRequested(Box::new(request)),
    })
}

fn resolved_frame(id: &str) -> UiJsonFrame {
    UiJsonFrame::Event(UiEventEnvelope {
        schema_version: ruse_types::SchemaVersion::current(),
        event_id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: ruse_types::Timestamp::now_utc(),
        conversation_id: ruse_types::ConversationId::new(),
        turn_id: None,
        task_id: None,
        correlation_id: ruse_types::CorrelationId::new(),
        payload: UiEventPayload::ApprovalResolved {
            request_id: id.to_string(),
            status: "approved".to_string(),
            decision: Some("allow".to_string()),
            reason: Some("approved from test".to_string()),
        },
    })
}

fn approval_queue_frame(requests: Vec<UiApprovalRequest>) -> UiJsonFrame {
    UiJsonFrame::ApprovalQueueSnapshot(
        approval_queue_snapshot_from_requests(requests).expect("approval queue snapshot"),
    )
}

#[test]
fn approval_panel_adds_pending_requests_and_moves_focus() {
    let mut panel = ApprovalPanelState::default();
    panel.reduce_frame(&approval_frame(approval_request(
        "00000000-0000-0000-0000-000000000001",
        "cargo test",
    )));
    panel.reduce_frame(&approval_frame(approval_request(
        "00000000-0000-0000-0000-000000000002",
        "cargo fmt",
    )));

    assert_eq!(panel.pending.len(), 2);
    assert_eq!(panel.focused_index, Some(1));

    assert_eq!(
        panel.reduce(ApprovalPanelAction::FocusPrevious),
        TuiEffect::None
    );
    assert_eq!(panel.focused_index, Some(0));
    assert_eq!(
        panel.reduce(ApprovalPanelAction::FocusNext),
        TuiEffect::None
    );
    assert_eq!(panel.focused_index, Some(1));
}

#[test]
fn approval_panel_reduces_queue_snapshot_and_renders_groups_and_lease() {
    let mut panel = ApprovalPanelState::default();
    panel.reduce_frame(&approval_queue_frame(vec![
        approval_request("00000000-0000-0000-0000-000000000002", "cargo fmt"),
        approval_request("00000000-0000-0000-0000-000000000001", "cargo test"),
    ]));

    assert_eq!(panel.pending.len(), 2);
    assert_eq!(panel.groups.len(), 1);
    assert_eq!(panel.groups[0].resource_kind, "command");
    assert_eq!(panel.focused_index, Some(0));

    let rendered = panel.render_text();
    assert!(rendered.contains("Approval queue: 2 pending"));
    assert!(rendered.contains("group command/execute/pending count=2"));
    assert!(rendered.contains("focus 1/2"));
    assert!(rendered.contains("lease=300s"));
    assert!(rendered.contains("redacted"));
}

#[test]
fn approval_panel_renders_empty_snapshot_with_expired_count() {
    let mut panel = ApprovalPanelState::default();
    panel.reduce_frame(&UiJsonFrame::ApprovalQueueSnapshot(
        UiApprovalQueueSnapshot {
            expired_count: 1,
            ..UiApprovalQueueSnapshot::default()
        },
    ));

    assert!(panel.pending.is_empty());
    assert_eq!(panel.expired_count, 1);
    assert_eq!(panel.focused_index, None);
    assert!(panel.is_active());
    assert_eq!(panel.render_text(), "Approval queue: empty\nExpired: 1");
}

#[test]
fn approval_panel_allow_and_deny_emit_resolve_approval_commands() {
    let mut panel = ApprovalPanelState::default();
    panel.reduce_frame(&approval_frame(approval_request(
        "00000000-0000-0000-0000-000000000001",
        "cargo test",
    )));

    let allow = panel.reduce(ApprovalPanelAction::Allow);
    let deny = panel.reduce(ApprovalPanelAction::Deny {
        reason: "too broad".to_string(),
    });

    let context = TuiBridgeContext::new(
        WorkspaceId::new(),
        ConversationId::new(),
        ClientKind::Tui,
        "ruse-tui-test",
    )
    .expect("valid context");
    let allow = allow
        .to_ui_command(&context)
        .expect("allow command")
        .expect("allow emits command");
    let deny = deny
        .to_ui_command(&context)
        .expect("deny command")
        .expect("deny emits command");

    assert!(matches!(
        allow.command,
        UiCommand::ResolveApproval {
            decision: UiApprovalDecision::Allow,
            reason,
            lease_duration_seconds: Some(300),
            ..
        } if reason == "approved from TUI"
    ));
    assert!(matches!(
        deny.command,
        UiCommand::ResolveApproval {
            decision: UiApprovalDecision::Deny,
            reason,
            lease_duration_seconds: None,
            ..
        } if reason == "too broad"
    ));
}

#[test]
fn approval_panel_rejects_missing_focus_and_empty_deny_reason() {
    let mut panel = ApprovalPanelState::default();

    assert_eq!(
        panel.reduce(ApprovalPanelAction::Allow),
        TuiEffect::LocalDiagnostic("no approval is focused".to_string())
    );

    panel.reduce_frame(&approval_frame(approval_request(
        "00000000-0000-0000-0000-000000000001",
        "cargo test",
    )));
    assert_eq!(
        panel.reduce(ApprovalPanelAction::Deny {
            reason: " ".to_string(),
        }),
        TuiEffect::LocalDiagnostic("deny requires a reason".to_string())
    );
}

#[test]
fn approval_panel_abort_and_resolved_frame_clear_pending_state() {
    let mut panel = ApprovalPanelState::default();
    panel.reduce_frame(&approval_frame(approval_request(
        "00000000-0000-0000-0000-000000000001",
        "cargo test",
    )));

    assert_eq!(
        panel.reduce(ApprovalPanelAction::Abort),
        TuiEffect::ApprovalAborted("00000000-0000-0000-0000-000000000001".to_string())
    );
    panel.reduce_frame(&resolved_frame("00000000-0000-0000-0000-000000000001"));

    assert!(panel.pending.is_empty());
    assert_eq!(panel.focused_index, None);
}

#[test]
fn app_reducer_populates_approval_panel_from_bridge_frames() {
    let mut app = TuiApp::default();

    app.reduce(ruse_tui::TuiAction::BridgeFrame(Box::new(approval_frame(
        approval_request("00000000-0000-0000-0000-000000000001", "cargo test"),
    ))));

    assert_eq!(app.approval_panel.pending.len(), 1);
    assert_eq!(app.approval_panel.focused_index, Some(0));

    let rendered = render_app_to_string(&app, 96, 20).expect("render app");
    assert!(rendered.contains("Approval"));
    assert!(rendered.contains("cargo test"));
    assert!(rendered.contains("redacted"));
}
