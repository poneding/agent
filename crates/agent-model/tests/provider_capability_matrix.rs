use std::sync::Arc;

use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelCapability,
    ModelCapabilityProfileError, ModelCostProfile, ModelEndpointConfig, ModelProviderConfig,
    ModelProviderKind, ModelRegistryError, ModelRequestRequirements, RegisteredModelProvider,
};
use serde_json::json;

fn fake_config(id: &str) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid fake config")
}

#[test]
fn provider_capability_matrix_serializes_stable_shape() {
    let capabilities = ModelCapabilities {
        streaming: true,
        tool_calls: true,
        structured_outputs: true,
        image_input: true,
        max_context_tokens: Some(128_000),
        max_output_tokens: Some(16_384),
        cost: Some(ModelCostProfile {
            input_per_million: Some("2.50".to_string()),
            output_per_million: Some("10.00".to_string()),
            currency: "USD".to_string(),
        }),
    };

    assert_eq!(
        serde_json::to_value(capabilities).expect("serialize capabilities"),
        json!({
            "streaming": true,
            "tool_calls": true,
            "structured_outputs": true,
            "image_input": true,
            "max_context_tokens": 128000,
            "max_output_tokens": 16384,
            "cost": {
                "input_per_million": "2.50",
                "output_per_million": "10.00",
                "currency": "USD"
            }
        })
    );
}

#[test]
fn provider_capability_matrix_deserializes_legacy_shape_with_defaults() {
    let value = json!({
        "streaming": true,
        "tool_calls": false,
        "structured_outputs": false,
        "max_context_tokens": 4096
    });

    let capabilities: ModelCapabilities =
        serde_json::from_value(value).expect("deserialize legacy capabilities");

    assert_eq!(
        capabilities,
        ModelCapabilities {
            streaming: true,
            tool_calls: false,
            structured_outputs: false,
            image_input: false,
            max_context_tokens: Some(4096),
            max_output_tokens: None,
            cost: None,
        }
    );
}

#[test]
fn provider_capability_matrix_default_profiles_are_deterministic() {
    let fake = ModelCapabilities::default_for_provider_kind(ModelProviderKind::Fake);
    let openai = ModelCapabilities::default_for_provider_kind(ModelProviderKind::OpenAiCompatible);

    assert_eq!(fake, ModelCapabilities::fake_default());
    assert_eq!(openai, ModelCapabilities::openai_compatible_default());
    assert_eq!(fake.max_context_tokens, Some(4096));
    assert_eq!(fake.max_output_tokens, Some(1024));
    assert!(!fake.image_input);
    assert_eq!(openai.max_context_tokens, Some(128_000));
    assert_eq!(openai.max_output_tokens, Some(16_384));
    assert!(!openai.image_input);
}

#[test]
fn provider_capability_matrix_reports_image_and_output_in_stable_order() {
    let capabilities = ModelCapabilities {
        streaming: false,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        max_context_tokens: Some(1024),
        max_output_tokens: Some(256),
        cost: None,
    };
    let requirements = ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: true,
        image_input: true,
        min_context_tokens: Some(8192),
        min_output_tokens: Some(2048),
    };

    assert_eq!(
        capabilities.missing_for(&requirements),
        vec![
            ModelCapability::Streaming,
            ModelCapability::ToolCalls,
            ModelCapability::StructuredOutputs,
            ModelCapability::ImageInput,
            ModelCapability::ContextTokens,
            ModelCapability::OutputTokens,
        ]
    );
    assert_eq!(
        capabilities.missing_labels_for(&requirements),
        vec![
            "streaming",
            "tool_calls",
            "structured_outputs",
            "image_input",
            "context_tokens",
            "output_tokens",
        ]
    );
}

#[test]
fn provider_capability_matrix_rejects_invalid_profile_before_registration() {
    let capabilities = ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        max_context_tokens: Some(4096),
        max_output_tokens: Some(0),
        cost: None,
    };

    let err = RegisteredModelProvider::new(
        fake_config("fake"),
        capabilities,
        Arc::new(FakeModelProvider::new(FakeModelScript::text("ready"))),
    )
    .expect_err("invalid capabilities rejected");

    assert!(matches!(
        err,
        ModelRegistryError::InvalidCapabilities {
            provider_id,
            source: ModelCapabilityProfileError::InvalidMaxOutputTokens { max_output_tokens: 0 }
        } if provider_id == "fake"
    ));
}
