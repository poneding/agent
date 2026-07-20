use ruse_tui::ExtensionPanelState;
use ruse_ui_bridge::{
    UiExtensionCapabilityReview, UiExtensionDiagnostic, UiExtensionSnapshot, UiExtensionSummary,
    UiJsonFrame,
};

fn snapshot() -> UiExtensionSnapshot {
    UiExtensionSnapshot::new(
        vec![
            UiExtensionSummary::new(
                "hello-plugin",
                "plugin",
                true,
                "healthy",
                Some("0.1.0".to_string()),
                "plugins/hello",
                1,
            )
            .unwrap(),
            UiExtensionSummary::new(
                "mcp-filesystem",
                "mcp_server",
                false,
                "unhealthy",
                None,
                "mcp filesystem",
                0,
            )
            .unwrap(),
        ],
        vec![
            UiExtensionCapabilityReview::new(
                "hello-plugin",
                "tool",
                "fs.read",
                "capability review is required",
            )
            .unwrap(),
        ],
        vec![
            UiExtensionDiagnostic::new(
                "error",
                "extension.unhealthy",
                "server failed",
                Some("mcp-filesystem".to_string()),
                false,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn extension_panel_reduces_snapshot_and_renders_health() {
    let mut panel = ExtensionPanelState::default();

    panel.reduce_frame(&UiJsonFrame::ExtensionSnapshot(snapshot()));

    assert_eq!(panel.snapshot.as_ref().unwrap().extensions.len(), 2);
    let rendered = panel.render_text();
    assert!(rendered.contains("hello-plugin"));
    assert!(rendered.contains("healthy"));
    assert!(rendered.contains("mcp-filesystem"));
    assert!(rendered.contains("extension.unhealthy"));
    assert!(rendered.contains("reviews=1"));
}

#[test]
fn extension_panel_renders_empty_state() {
    let panel = ExtensionPanelState::default();

    assert_eq!(panel.render_text(), "No extensions");
}
