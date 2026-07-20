use ruse_desktop::{DesktopExtensionHealth, DesktopExtensionStatus};
use ruse_ui_bridge::{
    UiExtensionCapabilityReview, UiExtensionDiagnostic, UiExtensionSnapshot, UiExtensionSummary,
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
                "broken-plugin",
                "plugin",
                false,
                "unhealthy",
                None,
                "plugins/broken",
                0,
            )
            .unwrap(),
        ],
        vec![
            UiExtensionCapabilityReview::new("hello-plugin", "tool", "fs.read", "review required")
                .unwrap(),
        ],
        vec![
            UiExtensionDiagnostic::new(
                "error",
                "extension.unhealthy",
                "token sk-secret password",
                Some("broken-plugin".to_string()),
                false,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn desktop_extension_health_counts_states_and_preserves_snapshot() {
    let health = DesktopExtensionHealth::new(snapshot()).unwrap();

    assert_eq!(health.status, DesktopExtensionStatus::NeedsAttention);
    assert_eq!(health.total_count, 2);
    assert_eq!(health.enabled_count, 1);
    assert_eq!(health.unhealthy_count, 1);
    assert_eq!(health.review_count, 1);
    assert_eq!(health.snapshot.extensions[0].id, "broken-plugin");
}

#[test]
fn desktop_extension_health_rejects_empty_extension_ids() {
    let mut snapshot = snapshot();
    snapshot.extensions[0].id = " ".to_string();

    let error = DesktopExtensionHealth::new(snapshot).unwrap_err();

    assert!(error.to_string().contains("extension id cannot be empty"));
}

#[test]
fn desktop_extension_health_does_not_expose_raw_secret_like_diagnostics() {
    let health = DesktopExtensionHealth::new(snapshot()).unwrap();
    let serialized = serde_json::to_string(&health).unwrap();

    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
    assert!(serialized.contains("[redacted]"));
}
