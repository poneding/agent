use ruse_ui_bridge::{
    UiConfigSnapshot, UiPlatformPathsSummary, UiSettingsProfileSummary, encode_json_frame,
};

#[test]
fn settings_paths_snapshot_serializes_optional_paths_and_settings() {
    let snapshot = UiConfigSnapshot::new("fake", "fake-scripted", Vec::new(), Vec::new())
        .expect("snapshot")
        .with_platform_paths(UiPlatformPathsSummary {
            config_dir: "/tmp/agent/config".to_string(),
            data_dir: "/tmp/agent/data".to_string(),
            cache_dir: "/tmp/agent/cache".to_string(),
            log_dir: "/tmp/agent/logs".to_string(),
            plugin_dir: "/tmp/agent/plugins".to_string(),
            skill_dir: "/tmp/agent/skills".to_string(),
            memory_dir: "/tmp/agent/memory".to_string(),
            temp_dir: "/tmp/agent/tmp".to_string(),
        })
        .with_settings(UiSettingsProfileSummary {
            profile_id: "default".to_string(),
            selected_provider: Some("fake".to_string()),
            selected_model: Some("fake-scripted".to_string()),
            policy_profile: "default".to_string(),
            release_channel: "dev".to_string(),
        });

    let value = serde_json::to_value(&snapshot).expect("serialize snapshot");

    assert_eq!(value["platform_paths"]["config_dir"], "/tmp/agent/config");
    assert_eq!(value["settings"]["release_channel"], "dev");
    let line = encode_json_frame(&ruse_ui_bridge::UiJsonFrame::ConfigSnapshot(snapshot))
        .expect("encode frame");
    assert_eq!(line.matches('\n').count(), 1);
}
