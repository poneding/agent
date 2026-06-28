use agent_desktop::{DesktopConfigSnapshot, DesktopError};
use agent_ui_bridge::{UiConfigSnapshot, UiModelProviderSummary};

fn provider() -> UiModelProviderSummary {
    UiModelProviderSummary::new("fake", "fake", "fake-scripted", true)
        .expect("valid provider summary")
}

#[test]
fn desktop_config_snapshot_rejects_empty_selection() {
    let empty_provider = UiConfigSnapshot {
        selected_provider: " ".to_string(),
        selected_model: "fake-scripted".to_string(),
        providers: vec![provider()],
        diagnostics: Vec::new(),
        platform_paths: None,
        settings: None,
    };
    let empty_model = UiConfigSnapshot {
        selected_provider: "fake".to_string(),
        selected_model: " ".to_string(),
        providers: vec![provider()],
        diagnostics: Vec::new(),
        platform_paths: None,
        settings: None,
    };

    let provider_err = DesktopConfigSnapshot::new(empty_provider).expect_err("empty provider");
    let model_err = DesktopConfigSnapshot::new(empty_model).expect_err("empty model");

    assert!(matches!(provider_err, DesktopError::InvalidSnapshot(_)));
    assert!(matches!(model_err, DesktopError::InvalidSnapshot(_)));
}

#[test]
fn desktop_config_snapshot_accepts_ui_bridge_snapshot() {
    let snapshot = UiConfigSnapshot::new("fake", "fake-scripted", vec![provider()], Vec::new())
        .expect("valid UI bridge snapshot");

    let desktop = DesktopConfigSnapshot::new(snapshot.clone()).expect("desktop snapshot");

    assert_eq!(desktop.snapshot, snapshot);
}
