use agent_desktop::{DesktopError, DesktopSettings, DesktopTheme, validate_desktop_settings};

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
