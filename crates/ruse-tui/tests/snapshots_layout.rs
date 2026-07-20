//! Whole-frame golden snapshots (via `insta`) at fixed sizes for representative
//! states. These lock the composited layout so a regression in spacing, wrapping,
//! overlay compositing, or the taxonomy is caught deterministically.
//!
//! Determinism: `render_app_to_string` renders through a `TestBackend`, and the
//! time-driven spinner reads pure `spinner_frame` / `elapsed_seconds` state (set
//! explicitly here), so no wall clock enters the render path. Every snapshot is
//! reproducible.

use ruse_tui::{
    StatusBarState, TranscriptItem, TranscriptItemKind, TuiAction, TuiApp, render_app_to_string,
};
use ruse_ui_bridge::{
    UiApprovalPreviewSummary, UiApprovalRequest, UiEventEnvelope, UiEventPayload, UiJsonFrame,
};

/// Fixed sizes from the guide: narrow, standard, wide.
const NARROW: (u16, u16) = (40, 10);
const STANDARD: (u16, u16) = (88, 24);
const WIDE: (u16, u16) = (120, 40);

fn demo_status() -> StatusBarState {
    StatusBarState::new(
        "openai",
        "gpt-5.5",
        "~/agent",
        "tui",
        "idle",
        "default-deny",
    )
    .with_context_total(Some(128_000))
}

/// A welcome-screen app (empty transcript), idle.
fn welcome_app() -> TuiApp {
    TuiApp::new(demo_status())
}

/// An app mid-turn with a streaming model answer. The spinner frame and elapsed
/// seconds are pinned so the activity line is deterministic.
fn streaming_app() -> TuiApp {
    let mut app = TuiApp::new(demo_status());
    app.mark_turn_started(std::time::Instant::now());
    // Pin the time-driven activity-line state so the snapshot never flickers.
    app.spinner_frame = 3;
    app.elapsed_seconds = 7;
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::User,
        "user",
        "explain the plan",
        false,
    ));
    app.transcript
        .push(TranscriptItem::streaming_model("Here is the plan so far"));
    app
}

/// An app showing the approval modal over a non-empty transcript, to prove the
/// `Clear` overlay removes transcript bleed-through.
fn approval_app() -> TuiApp {
    let mut app = TuiApp::new(demo_status());
    // Some transcript content behind the modal.
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::Model,
        "model",
        "I will run the test suite now.",
        false,
    ));
    let request = UiApprovalRequest {
        request_id: "00000000-0000-0000-0000-000000000001".to_string(),
        status: "pending".to_string(),
        action: "execute".to_string(),
        resource_kind: "command".to_string(),
        resource: "cargo test --all".to_string(),
        reason: "run the suite".to_string(),
        preview: UiApprovalPreviewSummary {
            kind: "command".to_string(),
            resource: "cargo test --all".to_string(),
            summary: "Run cargo test --all".to_string(),
            content_hash: None,
            truncated: false,
            redacted: false,
        },
        max_duration_seconds: None,
        expires_at: None,
    };
    let frame = UiJsonFrame::Event(UiEventEnvelope {
        schema_version: ruse_types::SchemaVersion::current(),
        event_id: ruse_types::EventId::new(),
        seq: None,
        occurred_at: ruse_types::Timestamp::now_utc(),
        conversation_id: ruse_types::ConversationId::new(),
        turn_id: None,
        task_id: None,
        correlation_id: ruse_types::CorrelationId::new(),
        payload: UiEventPayload::ApprovalRequested(Box::new(request)),
    });
    app.reduce(TuiAction::BridgeFrame(Box::new(frame)));
    app
}

fn snapshot(app: &TuiApp, (width, height): (u16, u16)) -> String {
    render_app_to_string(app, width, height).expect("render app to string")
}

#[test]
fn welcome_narrow() {
    insta::assert_snapshot!(snapshot(&welcome_app(), NARROW));
}

#[test]
fn welcome_standard() {
    insta::assert_snapshot!(snapshot(&welcome_app(), STANDARD));
}

#[test]
fn welcome_wide() {
    insta::assert_snapshot!(snapshot(&welcome_app(), WIDE));
}

#[test]
fn streaming_standard() {
    insta::assert_snapshot!(snapshot(&streaming_app(), STANDARD));
}

#[test]
fn streaming_narrow() {
    insta::assert_snapshot!(snapshot(&streaming_app(), NARROW));
}

#[test]
fn approval_modal_standard() {
    insta::assert_snapshot!(snapshot(&approval_app(), STANDARD));
}

#[test]
fn approval_modal_narrow() {
    // Below the 60-col side-panel cutoff: the modal must still be fully visible.
    insta::assert_snapshot!(snapshot(&approval_app(), NARROW));
}

/// Wide-CJK case: full-width characters must not break the layout.
#[test]
fn streaming_wide_cjk() {
    let mut app = TuiApp::new(demo_status());
    app.mark_turn_started(std::time::Instant::now());
    app.spinner_frame = 5;
    app.elapsed_seconds = 12;
    app.transcript.push(TranscriptItem::new(
        TranscriptItemKind::User,
        "user",
        "请解释这个项目的架构和主要模块",
        false,
    ));
    app.transcript.push(TranscriptItem::streaming_model(
        "这个项目分为多个 crate：终端界面、核心运行时、桥接层。",
    ));
    insta::assert_snapshot!(snapshot(&app, WIDE));
}
