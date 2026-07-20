use ruse_ui_bridge::{
    UiConfigDiagnostic, UiConfigDiagnosticSeverity, UiConfigSnapshot, UiJsonFrame,
    UiModelProviderCapabilities, UiModelProviderSummary, encode_json_frame,
};
use serde_json::json;

fn capabilities() -> UiModelProviderCapabilities {
    UiModelProviderCapabilities::new(true, false, false, false, Some(4096), Some(1024), None)
}

fn provider(id: &str, kind: &str, model: &str, enabled: bool) -> UiModelProviderSummary {
    UiModelProviderSummary::new(id, kind, model, enabled, capabilities())
        .expect("valid provider summary")
}

fn diagnostic() -> UiConfigDiagnostic {
    UiConfigDiagnostic::new(
        UiConfigDiagnosticSeverity::Warning,
        "config.layer.provider_overridden",
        "provider fake was overridden",
        Some("model.providers.fake".to_string()),
        Some("workspace".to_string()),
    )
    .expect("valid diagnostic")
}

#[test]
fn config_snapshot_serializes_stable_shape() {
    let snapshot = UiConfigSnapshot::new(
        "openai",
        "gpt-4.1-mini",
        vec![
            provider("openai", "open_ai_compatible", "gpt-4.1-mini", true),
            provider("fake", "fake", "fake-scripted", true),
        ],
        vec![diagnostic()],
    )
    .expect("valid config snapshot");

    let value = serde_json::to_value(snapshot).expect("serialize snapshot");

    assert_eq!(
        value,
        json!({
            "selected_provider": "openai",
            "selected_model": "gpt-4.1-mini",
            "providers": [
                {
                    "id": "fake",
                    "kind": "fake",
                    "default_model": "fake-scripted",
                    "enabled": true,
                    "capabilities": {
                        "streaming": true,
                        "tool_calls": false,
                        "structured_outputs": false,
                        "image_input": false,
                        "max_context_tokens": 4096,
                        "max_output_tokens": 1024,
                        "cost": null
                    }
                },
                {
                    "id": "openai",
                    "kind": "open_ai_compatible",
                    "default_model": "gpt-4.1-mini",
                    "enabled": true,
                    "capabilities": {
                        "streaming": true,
                        "tool_calls": false,
                        "structured_outputs": false,
                        "image_input": false,
                        "max_context_tokens": 4096,
                        "max_output_tokens": 1024,
                        "cost": null
                    }
                }
            ],
            "provider_support": [],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "config.layer.provider_overridden",
                    "message": "provider fake was overridden",
                    "field_path": "model.providers.fake",
                    "source": "workspace"
                }
            ]
        })
    );
}

#[test]
fn config_snapshot_keeps_disabled_providers_visible() {
    let snapshot = UiConfigSnapshot::new(
        "fake",
        "fake-scripted",
        vec![
            provider("openai", "open_ai_compatible", "gpt-4.1-mini", false),
            provider("fake", "fake", "fake-scripted", true),
        ],
        Vec::new(),
    )
    .expect("valid config snapshot");

    let disabled = snapshot
        .providers
        .iter()
        .find(|provider| provider.id == "openai")
        .expect("disabled provider remains visible");

    assert!(!disabled.enabled);
}

#[test]
fn config_snapshot_redacts_secret_like_values() {
    let snapshot = UiConfigSnapshot::new(
        "openai",
        "gpt-4.1-mini",
        vec![provider(
            "openai",
            "open_ai_compatible",
            "gpt-4.1-mini",
            true,
        )],
        Vec::new(),
    )
    .expect("valid config snapshot");

    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");

    assert!(!serialized.contains("OPENAI_API_KEY"));
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("api_token"));
    assert!(!serialized.contains("password"));
    assert!(!serialized.contains("secret"));
}

#[test]
fn config_snapshot_json_frame_is_single_line() {
    let snapshot = UiConfigSnapshot::new(
        "fake",
        "fake-scripted",
        vec![provider("fake", "fake", "fake-scripted", true)],
        Vec::new(),
    )
    .expect("valid config snapshot");

    let line = encode_json_frame(&UiJsonFrame::ConfigSnapshot(snapshot)).expect("encode frame");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::ConfigSnapshot(_)));
}
