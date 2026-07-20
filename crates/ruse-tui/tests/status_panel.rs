use ruse_tui::{StatusBarState, StatusPanelState};
use ruse_ui_bridge::{UiJsonFrame, UiTerminalStatusSnapshot};

#[test]
fn status_panel_renders_status_bar_snapshot() {
    let status = StatusBarState::new(
        "fake",
        "fake-scripted",
        ".",
        "tui",
        "connected",
        "default-deny",
    );

    let panel = StatusPanelState::from_status_bar(&status, "status", 2, 1);

    assert_eq!(panel.snapshot.provider, "fake");
    let rendered = panel.render_text();
    assert!(rendered.contains("Provider: fake / fake-scripted"));
    assert!(rendered.contains("Workspace: ."));
    assert!(rendered.contains("Sessions: 2"));
    assert!(rendered.contains("Diagnostics: 1"));
}

#[test]
fn status_panel_reduces_bridge_status_snapshot() {
    let mut panel = StatusPanelState::default();
    let snapshot = UiTerminalStatusSnapshot::new(
        "provider-secret",
        "model-token",
        "/tmp/api_key/project",
        "tui",
        "connected",
        "sessions",
        1,
        0,
    );

    panel.reduce_frame(&UiJsonFrame::TerminalStatusSnapshot(snapshot));

    let rendered = panel.render_text();
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("provider-secret"));
    assert!(!rendered.contains("model-token"));
    assert!(!rendered.contains("api_key"));
}
