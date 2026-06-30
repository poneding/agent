use std::sync::Arc;

use agent_cli::providers::build_model_provider_registry;
use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, ConfigOverride,
    RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    EnvSecretResolver, InMemorySecretResolver, ModelEndpointConfig, ModelFallbackChain,
    ModelInvocationPolicy, ModelProviderCatalog, ModelProviderConfig, ModelProviderKind,
    ModelRequestRequirements, ModelRetryPolicy, SecretRef,
};

fn fake_provider(id: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        "fake-scripted",
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        enabled,
    )
    .expect("valid fake provider")
}

fn openai_provider(id: &str, secret_name: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::OpenAiCompatible,
        "gpt-4.1-mini",
        ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(SecretRef::new("env", secret_name).expect("valid secret ref")),
        enabled,
    )
    .expect("valid OpenAI-compatible provider")
}

fn anthropic_provider(id: &str, secret_name: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Anthropic,
        "claude-3-5-sonnet-latest",
        ModelEndpointConfig::new(Some("https://api.anthropic.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(SecretRef::new("env", secret_name).expect("valid secret ref")),
        enabled,
    )
    .expect("valid Anthropic provider")
}

fn gemini_provider(id: &str, secret_name: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Gemini,
        "gemini-1.5-pro",
        ModelEndpointConfig::new(
            Some("https://generativelanguage.googleapis.com/v1beta"),
            30_000,
        )
        .expect("valid endpoint"),
        Some(SecretRef::new("env", secret_name).expect("valid secret ref")),
        enabled,
    )
    .expect("valid Gemini provider")
}

fn invocation_policy() -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
    )
    .expect("valid invocation policy")
}

fn effective_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> agent_core::EffectiveRuntimeConfig {
    build_effective_runtime_config(
        vec![ConfigLayer {
            kind: ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            config: AgentRuntimeConfig {
                schema_version: AGENT_CONFIG_SCHEMA_VERSION,
                model: RuntimeModelConfig {
                    default_provider: default_provider.to_string(),
                    catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
                    invocation_policy: invocation_policy(),
                },
            },
        }],
        ConfigOverride::default(),
    )
    .expect("effective config")
}

fn prompt_requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        image_input: false,
        min_context_tokens: Some(1024),
        min_output_tokens: Some(1024),
    }
}

#[test]
fn live_provider_registration_registers_enabled_config_providers() {
    let secret_ref = SecretRef::new("env", "AGENT_TEST_OPENAI_API_KEY").expect("secret ref");
    let anthropic_secret_ref =
        SecretRef::new("env", "AGENT_TEST_ANTHROPIC_API_KEY").expect("secret ref");
    let gemini_secret_ref = SecretRef::new("env", "AGENT_TEST_GEMINI_API_KEY").expect("secret ref");
    let resolver = InMemorySecretResolver::new()
        .with_secret(secret_ref, "sk-test-secret")
        .expect("secret resolver")
        .with_secret(anthropic_secret_ref, "sk-ant-test-secret")
        .expect("secret resolver")
        .with_secret(gemini_secret_ref, "sk-gemini-test-secret")
        .expect("secret resolver");
    let effective = effective_config(
        "fake",
        vec![
            fake_provider("fake", true),
            openai_provider("openai", "AGENT_TEST_OPENAI_API_KEY", true),
            anthropic_provider("anthropic", "AGENT_TEST_ANTHROPIC_API_KEY", true),
            gemini_provider("gemini", "AGENT_TEST_GEMINI_API_KEY", true),
        ],
    );

    let registry = build_model_provider_registry(&effective, "hello", Arc::new(resolver))
        .expect("registry builds");

    assert_eq!(registry.len(), 4);
    assert_eq!(
        registry.get("fake").expect("fake provider").config.id,
        "fake"
    );
    assert_eq!(
        registry
            .require_capabilities("openai", &prompt_requirements())
            .expect("openai provider")
            .config
            .id,
        "openai"
    );
    assert_eq!(
        registry
            .require_capabilities("anthropic", &prompt_requirements())
            .expect("anthropic provider")
            .config
            .id,
        "anthropic"
    );
    assert_eq!(
        registry
            .require_capabilities("gemini", &prompt_requirements())
            .expect("gemini provider")
            .config
            .id,
        "gemini"
    );
}

#[test]
fn live_provider_registration_skips_disabled_providers() {
    let effective = effective_config(
        "fake",
        vec![
            fake_provider("fake", true),
            openai_provider("openai", "AGENT_TEST_DISABLED_OPENAI_API_KEY", false),
        ],
    );

    let registry =
        build_model_provider_registry(&effective, "hello", Arc::new(InMemorySecretResolver::new()))
            .expect("registry builds");

    assert_eq!(registry.len(), 1);
    assert!(registry.get("fake").is_ok());
    assert!(registry.get("openai").is_err());
}

#[test]
fn live_provider_registration_missing_env_secret_fails_before_network() {
    let missing_name = format!(
        "AGENT_TEST_M17_MISSING_OPENAI_API_KEY_{}",
        std::process::id()
    );
    let effective = effective_config(
        "openai",
        vec![
            fake_provider("fake", true),
            openai_provider("openai", &missing_name, true),
        ],
    );

    let error = build_model_provider_registry(&effective, "hello", Arc::new(EnvSecretResolver))
        .expect_err("missing secret should fail at registration");
    let rendered = error.to_string();

    assert!(rendered.contains("prompt runtime failed"));
    assert!(rendered.contains("openai"));
    assert!(rendered.contains("secret"));
    assert!(!rendered.contains(&missing_name));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}

#[test]
fn live_provider_registration_missing_anthropic_env_secret_fails_before_network() {
    let missing_name = format!(
        "AGENT_TEST_M57_MISSING_ANTHROPIC_API_KEY_{}",
        std::process::id()
    );
    let effective = effective_config(
        "anthropic",
        vec![
            fake_provider("fake", true),
            anthropic_provider("anthropic", &missing_name, true),
        ],
    );

    let error = build_model_provider_registry(&effective, "hello", Arc::new(EnvSecretResolver))
        .expect_err("missing secret should fail at registration");
    let rendered = error.to_string();

    assert!(rendered.contains("prompt runtime failed"));
    assert!(rendered.contains("anthropic"));
    assert!(rendered.contains("secret"));
    assert!(!rendered.contains(&missing_name));
    assert!(!rendered.contains("sk-ant-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}

#[test]
fn live_provider_registration_missing_gemini_env_secret_fails_before_network() {
    let missing_name = format!(
        "AGENT_TEST_M57_MISSING_GEMINI_API_KEY_{}",
        std::process::id()
    );
    let effective = effective_config(
        "gemini",
        vec![
            fake_provider("fake", true),
            gemini_provider("gemini", &missing_name, true),
        ],
    );

    let error = build_model_provider_registry(&effective, "hello", Arc::new(EnvSecretResolver))
        .expect_err("missing secret should fail at registration");
    let rendered = error.to_string();

    assert!(rendered.contains("prompt runtime failed"));
    assert!(rendered.contains("gemini"));
    assert!(rendered.contains("secret"));
    assert!(!rendered.contains(&missing_name));
    assert!(!rendered.contains("sk-gemini-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}
