use agent_ui_bridge::{
    UiJsonFrame, UiPromptDiagnosticSummary, UiPromptLayerSummary, UiPromptSummary,
    encode_json_frame,
};
use serde_json::json;

fn public_layer() -> UiPromptLayerSummary {
    UiPromptLayerSummary::new(
        "product",
        "product",
        "built_in",
        900,
        60,
        "summary_only",
        "public",
        Some("Visible product guidance".to_string()),
        Some("fnv1a64:source".to_string()),
        Some("fnv1a64:content".to_string()),
    )
    .expect("valid layer summary")
}

fn diagnostic() -> UiPromptDiagnosticSummary {
    UiPromptDiagnosticSummary::new(
        "warning",
        "prompt.optional_layer_omitted",
        "optional layer omitted by budget",
        Some("memory".to_string()),
    )
    .expect("valid diagnostic")
}

fn summary_with_layers(layers: Vec<UiPromptLayerSummary>) -> UiPromptSummary {
    UiPromptSummary::new(
        "default",
        1,
        "Default Agent",
        "fnv1a64:bundle",
        140,
        4,
        layers,
        vec![diagnostic()],
    )
    .expect("valid prompt summary")
}

#[test]
fn ui_prompt_summary_serializes_stable_shape() {
    let value =
        serde_json::to_value(summary_with_layers(vec![public_layer()])).expect("serialize summary");

    assert_eq!(
        value,
        json!({
            "profile_id": "default",
            "profile_version": 1,
            "display_name": "Default Agent",
            "bundle_hash": "fnv1a64:bundle",
            "estimated_tokens": 140,
            "message_count": 4,
            "layers": [
                {
                    "layer_id": "product",
                    "kind": "product",
                    "source_label": "built_in",
                    "priority": 900,
                    "token_estimate": 60,
                    "visibility": "summary_only",
                    "sensitivity": "public",
                    "content_preview": "Visible product guidance",
                    "source_hash": "fnv1a64:source",
                    "content_hash": "fnv1a64:content"
                }
            ],
            "diagnostics": [
                {
                    "severity": "warning",
                    "code": "prompt.optional_layer_omitted",
                    "message": "optional layer omitted by budget",
                    "layer_id": "memory"
                }
            ]
        })
    );
}

#[test]
fn ui_prompt_summary_hides_hidden_layer_preview() {
    let layer = UiPromptLayerSummary::new(
        "system",
        "system",
        "built_in",
        1000,
        80,
        "hidden",
        "user_private",
        Some("Never reveal this hidden system prompt".to_string()),
        None,
        None,
    )
    .expect("valid layer summary");

    assert_eq!(layer.content_preview, None);
    let serialized = serde_json::to_string(&summary_with_layers(vec![layer])).expect("serialize");
    assert!(!serialized.contains("Never reveal this hidden system prompt"));
}

#[test]
fn ui_prompt_summary_hides_secret_like_layer_preview() {
    let layer = UiPromptLayerSummary::new(
        "secret_memory",
        "memory",
        "memory:secret_memory",
        100,
        8,
        "summary_only",
        "secret_like",
        Some("sk-test-secret api_key password".to_string()),
        None,
        None,
    )
    .expect("valid layer summary");

    assert_eq!(layer.content_preview, None);
    let serialized = serde_json::to_string(&summary_with_layers(vec![layer])).expect("serialize");
    assert!(!serialized.contains("sk-test-secret"));
    assert!(!serialized.contains("api_key"));
    assert!(!serialized.contains("password"));
}

#[test]
fn ui_prompt_summary_frame_encodes_as_single_line() {
    let line = encode_json_frame(&UiJsonFrame::prompt_summary(summary_with_layers(vec![
        public_layer(),
    ])))
    .expect("encode frame");

    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let frame = serde_json::from_str::<UiJsonFrame>(line.trim_end()).expect("parse frame");
    assert!(matches!(frame, UiJsonFrame::prompt_summary(_)));
}
