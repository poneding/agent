use std::path::PathBuf;

use ruse_core::ReleaseChannel;
use ruse_desktop::{
    DesktopError, DesktopSettings, DesktopSettingsDiagnosticSeverity, DesktopTheme,
    build_desktop_release_smoke_snapshot, build_provider_settings_summary,
    validate_desktop_settings,
};
use ruse_ui_bridge::{
    UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot, UiModelProviderCapabilities,
    UiModelProviderSummary,
};

fn provider(enabled: bool) -> UiModelProviderSummary {
    UiModelProviderSummary::new(
        "fake",
        "fake",
        "fake-scripted",
        enabled,
        UiModelProviderCapabilities::new(true, true, false, false, Some(4096), Some(1024), None),
    )
    .expect("valid provider")
}

fn config_snapshot(message: &str) -> UiConfigSnapshot {
    UiConfigSnapshot::new(
        "fake",
        "fake-scripted",
        vec![provider(true)],
        vec![
            UiConfigDiagnostic::new(
                UiConfigDiagnosticSeverity::Warning,
                "config.fixture",
                message,
                Some("model.provider".to_string()),
                Some("test".to_string()),
            )
            .expect("valid diagnostic"),
        ],
    )
    .expect("valid config snapshot")
}

#[test]
fn settings_serializes_stable_shape() {
    let settings = DesktopSettings::default();

    let value = serde_json::to_value(settings).expect("serialize settings");

    assert_eq!(value["model"]["provider"], "default");
    assert_eq!(value["model"]["model"], "default");
    assert_eq!(value["policy"]["default_decision"], "ask");
    assert_eq!(value["policy"]["require_approval_for_shell"], true);
    assert_eq!(value["memory"]["project_memory_enabled"], false);
    assert_eq!(value["memory"]["long_term_memory_enabled"], false);
    assert_eq!(value["mcp"]["enabled"], false);
    assert_eq!(value["mcp"]["server_count"], 0);
    assert_eq!(value["ui"]["theme"], "system");
    assert_eq!(value["ui"]["show_trace_timeline"], true);
}

#[test]
fn settings_defaults_to_conservative_policy() {
    let settings = DesktopSettings::default();

    assert_eq!(settings.policy.default_decision, "ask");
    assert!(settings.policy.require_approval_for_shell);
    assert!(!settings.mcp.enabled);
    assert_eq!(settings.ui.theme, DesktopTheme::System);
}

#[test]
fn settings_rejects_empty_provider_and_model() {
    let mut empty_provider = DesktopSettings::default();
    empty_provider.model.provider = " ".to_string();
    let mut empty_model = DesktopSettings::default();
    empty_model.model.model = " ".to_string();

    let provider_err = validate_desktop_settings(&empty_provider).unwrap_err();
    let model_err = validate_desktop_settings(&empty_model).unwrap_err();

    assert!(matches!(provider_err, DesktopError::InvalidSettings(_)));
    assert!(matches!(model_err, DesktopError::InvalidSettings(_)));
}

#[test]
fn settings_rejects_invalid_policy_decision() {
    let mut settings = DesktopSettings::default();
    settings.policy.default_decision = "always_allow".to_string();

    let err = validate_desktop_settings(&settings).unwrap_err();

    assert!(matches!(err, DesktopError::InvalidSettings(_)));
}

#[test]
fn settings_serialized_shape_excludes_secret_like_keys() {
    let settings = DesktopSettings::default();

    let serialized = serde_json::to_string(&settings).expect("serialize settings");

    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("apikey"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("secret"));
    assert!(!serialized.contains("env"));
}

#[test]
fn settings_release_smoke_snapshot_uses_portable_root() {
    let root = std::env::temp_dir().join(format!(
        "ruse-desktop-release-smoke-{}",
        std::process::id()
    ));

    let snapshot = build_desktop_release_smoke_snapshot("0.1.0", ReleaseChannel::Dev, root.clone())
        .expect("release smoke snapshot");

    assert_eq!(snapshot.app_version, "0.1.0");
    assert_eq!(snapshot.release_channel, ReleaseChannel::Dev);
    assert_eq!(snapshot.config_dir, root.join("config"));
    assert_eq!(snapshot.data_dir, root.join("data"));
    assert_eq!(snapshot.cache_dir, root.join("cache"));
    assert_eq!(snapshot.log_dir, root.join("logs"));
    assert_eq!(snapshot.settings_profile_id, "default");
}

#[test]
fn settings_release_smoke_snapshot_rejects_relative_root_without_side_effects() {
    let error = build_desktop_release_smoke_snapshot(
        "0.1.0",
        ReleaseChannel::Dev,
        PathBuf::from("relative-smoke-root"),
    )
    .expect_err("relative root rejected");

    assert!(matches!(error, DesktopError::InvalidSettings(_)));
}

#[test]
fn settings_builds_provider_summary_with_capabilities() {
    let config = ruse_desktop::DesktopConfigSnapshot::new(config_snapshot("using fake provider"))
        .expect("desktop config");
    let mut settings = DesktopSettings::default();
    settings.model.provider = "fake".to_string();
    settings.model.model = "fake-scripted".to_string();

    let summary = build_provider_settings_summary(&config, &settings).expect("provider summary");

    assert_eq!(summary.selected_provider, "fake");
    assert_eq!(summary.selected_model, "fake-scripted");
    assert_eq!(summary.providers[0].id, "fake");
    assert!(summary.providers[0].enabled);
    assert!(summary.providers[0].supports_tools);
    assert!(summary.providers[0].supports_streaming);
    assert_eq!(summary.providers[0].context_window, Some(4096));
    assert_eq!(summary.providers[0].output_limit, Some(1024));
    assert!(summary.validation.valid);
    assert_eq!(summary.validation.diagnostic_count, 1);
}

#[test]
fn settings_provider_summary_redacts_diagnostics_and_validation_errors() {
    let config = ruse_desktop::DesktopConfigSnapshot::new(config_snapshot(
        "api_key sk-secret token password should not leak",
    ))
    .expect("desktop config");
    let mut settings = DesktopSettings::default();
    settings.model.provider = " ".to_string();

    let summary = build_provider_settings_summary(&config, &settings).expect("provider summary");
    let serialized = serde_json::to_string(&summary).expect("serialize provider summary");

    assert!(!summary.validation.valid);
    assert!(summary.validation.diagnostic_count >= 2);
    assert!(
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DesktopSettingsDiagnosticSeverity::Error)
    );
    assert!(serialized.contains("[redacted]"));
    assert!(!serialized.contains("sk-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("password"));
}
