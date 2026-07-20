use ruse_ui_bridge::{
    UiExtensionCapabilityReview, UiExtensionDiagnostic, UiExtensionSnapshot, UiExtensionSummary,
    UiJsonFrame, encode_json_frame,
};

#[test]
fn extension_snapshot_sorts_summaries_and_serializes_stable_shape() {
    let plugin = UiExtensionSummary::new(
        "hello-plugin",
        "plugin",
        true,
        "healthy",
        Some("0.1.0".to_string()),
        "plugins/hello",
        2,
    )
    .unwrap();
    let mcp = UiExtensionSummary::new(
        "filesystem",
        "mcp_server",
        false,
        "unhealthy",
        None,
        "mcp filesystem",
        1,
    )
    .unwrap();

    let snapshot = UiExtensionSnapshot::new(
        vec![mcp, plugin],
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
                "warning",
                "extension.unhealthy",
                "server reported [redacted]",
                Some("filesystem".to_string()),
                false,
            )
            .unwrap(),
        ],
    )
    .unwrap();

    assert_eq!(snapshot.extensions[0].id, "filesystem");
    assert_eq!(snapshot.extensions[1].id, "hello-plugin");
    let value = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(value["extensions"][1]["capability_count"], 2);
    assert_eq!(
        value["capability_reviews"][0]["extension_id"],
        "hello-plugin"
    );
}

#[test]
fn extension_snapshot_redacts_secret_like_values() {
    let snapshot = UiExtensionSnapshot::new(
        vec![
            UiExtensionSummary::new(
                "secret-plugin",
                "plugin",
                true,
                "healthy",
                None,
                "/tmp/sk-secret-token",
                0,
            )
            .unwrap(),
        ],
        Vec::new(),
        vec![
            UiExtensionDiagnostic::new(
                "error",
                "extension.secret",
                "token sk-secret password api_key",
                Some("secret-plugin".to_string()),
                false,
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("api_key"));
    assert!(serialized.contains("[redacted]"));
}

#[test]
fn extension_snapshot_json_frame_is_single_line() {
    let snapshot = UiExtensionSnapshot::new(Vec::new(), Vec::new(), Vec::new()).unwrap();

    let line = encode_json_frame(&UiJsonFrame::extension_snapshot(snapshot)).unwrap();
    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();

    assert!(line.ends_with('\n'));
    assert_eq!(value["type"], "extension_snapshot");
}
