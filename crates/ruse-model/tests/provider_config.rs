use ruse_model::{
    ModelConfigError, ModelEndpointConfig, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, SecretRef,
};
use serde_json::json;

fn api_key_ref() -> SecretRef {
    SecretRef::new("env", "OPENAI_API_KEY").expect("valid secret ref")
}

fn anthropic_api_key_ref() -> SecretRef {
    SecretRef::new("env", "ANTHROPIC_API_KEY").expect("valid secret ref")
}

fn gemini_api_key_ref() -> SecretRef {
    SecretRef::new("env", "GEMINI_API_KEY").expect("valid secret ref")
}

#[test]
fn provider_config_serializes_stable_shape() {
    let config = ModelProviderConfig::new(
        "openai",
        ModelProviderKind::OpenAiCompatible,
        "gpt-4.1-mini",
        ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(api_key_ref()),
        true,
    )
    .expect("valid provider config");

    assert_eq!(
        serde_json::to_value(config).expect("serialize provider config"),
        json!({
            "id": "openai",
            "kind": "open_ai_compatible",
            "default_model": "gpt-4.1-mini",
            "endpoint": {
                "base_url": "https://api.openai.com/v1",
                "timeout_ms": 30000
            },
            "api_key": {
                "provider": "env",
                "name": "OPENAI_API_KEY"
            },
            "enabled": true
        })
    );
}

#[test]
fn provider_config_requires_secret_ref_for_openai_compatible() {
    let err = ModelProviderConfig::new(
        "openai",
        ModelProviderKind::OpenAiCompatible,
        "gpt-4.1-mini",
        ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        ModelConfigError::MissingSecretRef { provider_id } if provider_id == "openai"
    ));
}

#[test]
fn provider_config_serializes_anthropic_kind() {
    let config = ModelProviderConfig::new(
        "anthropic",
        ModelProviderKind::Anthropic,
        "claude-3-5-sonnet-latest",
        ModelEndpointConfig::new(Some("https://api.anthropic.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(anthropic_api_key_ref()),
        true,
    )
    .expect("valid provider config");

    assert_eq!(
        serde_json::to_value(config).expect("serialize provider config"),
        json!({
            "id": "anthropic",
            "kind": "anthropic",
            "default_model": "claude-3-5-sonnet-latest",
            "endpoint": {
                "base_url": "https://api.anthropic.com/v1",
                "timeout_ms": 30000
            },
            "api_key": {
                "provider": "env",
                "name": "ANTHROPIC_API_KEY"
            },
            "enabled": true
        })
    );
}

#[test]
fn provider_config_requires_secret_ref_for_anthropic() {
    let err = ModelProviderConfig::new(
        "anthropic",
        ModelProviderKind::Anthropic,
        "claude-3-5-sonnet-latest",
        ModelEndpointConfig::new(Some("https://api.anthropic.com/v1"), 30_000)
            .expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        ModelConfigError::MissingSecretRef { provider_id } if provider_id == "anthropic"
    ));
}

#[test]
fn provider_config_serializes_gemini_kind() {
    let config = ModelProviderConfig::new(
        "gemini",
        ModelProviderKind::Gemini,
        "gemini-1.5-pro",
        ModelEndpointConfig::new(
            Some("https://generativelanguage.googleapis.com/v1beta"),
            30_000,
        )
        .expect("valid endpoint"),
        Some(gemini_api_key_ref()),
        true,
    )
    .expect("valid provider config");

    assert_eq!(
        serde_json::to_value(config).expect("serialize provider config"),
        json!({
            "id": "gemini",
            "kind": "gemini",
            "default_model": "gemini-1.5-pro",
            "endpoint": {
                "base_url": "https://generativelanguage.googleapis.com/v1beta",
                "timeout_ms": 30000
            },
            "api_key": {
                "provider": "env",
                "name": "GEMINI_API_KEY"
            },
            "enabled": true
        })
    );
}

#[test]
fn provider_config_requires_secret_ref_for_gemini() {
    let err = ModelProviderConfig::new(
        "gemini",
        ModelProviderKind::Gemini,
        "gemini-1.5-pro",
        ModelEndpointConfig::new(
            Some("https://generativelanguage.googleapis.com/v1beta"),
            30_000,
        )
        .expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("missing secret ref rejected");

    assert!(matches!(
        err,
        ModelConfigError::MissingSecretRef { provider_id } if provider_id == "gemini"
    ));
}

#[test]
fn provider_config_serializes_local_kind_without_secret() {
    let config = ModelProviderConfig::new(
        "local",
        ModelProviderKind::Local,
        "llama3.1",
        ModelEndpointConfig::new(Some("http://localhost:11434"), 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider config");

    assert_eq!(
        serde_json::to_value(config).expect("serialize provider config"),
        json!({
            "id": "local",
            "kind": "local",
            "default_model": "llama3.1",
            "endpoint": {
                "base_url": "http://localhost:11434",
                "timeout_ms": 30000
            },
            "api_key": null,
            "enabled": true
        })
    );
}

#[test]
fn provider_config_requires_local_base_url_for_local_provider() {
    let err = ModelProviderConfig::new(
        "local",
        ModelProviderKind::Local,
        "llama3.1",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("missing local endpoint rejected");

    assert!(matches!(
        err,
        ModelConfigError::MissingEndpoint { provider_id } if provider_id == "local"
    ));
}

#[test]
fn provider_config_rejects_remote_endpoint_for_local_provider() {
    let err = ModelProviderConfig::new(
        "local",
        ModelProviderKind::Local,
        "llama3.1",
        ModelEndpointConfig::new(Some("https://api.example.test"), 30_000)
            .expect("generic https endpoint is syntactically valid"),
        None,
        true,
    )
    .expect_err("remote local endpoint rejected");

    assert!(matches!(err, ModelConfigError::UnsafeEndpointUrl { .. }));
}

#[test]
fn provider_config_rejects_invalid_ids_models_and_timeouts() {
    let invalid_id = ModelProviderConfig::new(
        "OpenAI",
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("uppercase provider id rejected");
    let empty_model = ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        " ",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect_err("empty model rejected");
    let invalid_timeout = ModelEndpointConfig::new(None, 0).expect_err("zero timeout rejected");

    assert!(matches!(
        invalid_id,
        ModelConfigError::InvalidProviderId { .. }
    ));
    assert!(matches!(
        empty_model,
        ModelConfigError::EmptyDefaultModel { provider_id } if provider_id == "fake"
    ));
    assert!(matches!(
        invalid_timeout,
        ModelConfigError::InvalidTimeout { timeout_ms: 0 }
    ));
}

#[test]
fn provider_config_rejects_unsafe_endpoint_urls() {
    let unsafe_http = ModelEndpointConfig::new(Some("http://api.example.test/v1"), 30_000)
        .expect_err("non-local http endpoint rejected");
    let unsupported_scheme = ModelEndpointConfig::new(Some("ftp://api.example.test/v1"), 30_000)
        .expect_err("unsupported endpoint scheme rejected");
    let local_http = ModelEndpointConfig::new(Some("http://localhost:11434/v1"), 30_000)
        .expect("local http endpoint allowed");

    assert!(matches!(
        unsafe_http,
        ModelConfigError::UnsafeEndpointUrl { .. }
    ));
    assert!(matches!(
        unsupported_scheme,
        ModelConfigError::UnsafeEndpointUrl { .. }
    ));
    assert_eq!(
        local_http.base_url.as_deref(),
        Some("http://localhost:11434/v1")
    );
}

#[test]
fn provider_catalog_rejects_duplicate_provider_ids() {
    let first = ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid fake provider");
    let duplicate = ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        "other-script",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid fake provider");

    let err = ModelProviderCatalog::new(vec![first, duplicate])
        .expect_err("duplicate provider id rejected");

    assert!(matches!(
        err,
        ModelConfigError::DuplicateProviderId { id } if id == "fake"
    ));
}

#[test]
fn provider_catalog_requires_enabled_provider() {
    let disabled = ModelProviderConfig::new(
        "fake",
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        false,
    )
    .expect("valid disabled fake provider");

    let err = ModelProviderCatalog::new(vec![disabled]).expect_err("enabled provider required");

    assert!(matches!(err, ModelConfigError::NoEnabledProviders));
}

#[test]
fn secret_ref_serializes_without_raw_secret_value() {
    let secret = api_key_ref();
    let value = serde_json::to_value(secret).expect("serialize secret ref");

    assert_eq!(
        value,
        json!({
            "provider": "env",
            "name": "OPENAI_API_KEY"
        })
    );
    assert!(value.get("value").is_none());
    assert!(value.get("token").is_none());
    assert!(value.get("api_key_value").is_none());
    assert!(!value.to_string().contains("sk-"));
}
