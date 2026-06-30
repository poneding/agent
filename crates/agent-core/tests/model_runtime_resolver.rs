use std::sync::Arc;

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigDiagnostic, ConfigLayerSummary,
    ConfigOverride, EffectiveRuntimeConfig, ModelRuntimeError, ModelRuntimeResolver,
    RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelCapability, ModelEndpointConfig,
    ModelFallbackChain, ModelInvocationPolicy, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequestRequirements, ModelRetryPolicy,
    RegisteredModelProvider, SecretRef,
};

fn capabilities(
    streaming: bool,
    tool_calls: bool,
    structured_outputs: bool,
    max_context_tokens: Option<u32>,
) -> ModelCapabilities {
    ModelCapabilities {
        streaming,
        tool_calls,
        structured_outputs,
        image_input: false,
        max_context_tokens,
        max_output_tokens: Some(1024),
        cost: None,
    }
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

fn invocation_policy() -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
    )
    .expect("valid invocation policy")
}

fn fake_provider(id: &str, model: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        model,
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        enabled,
    )
    .expect("valid fake provider")
}

fn openai_provider() -> ModelProviderConfig {
    ModelProviderConfig::new(
        "openai",
        ModelProviderKind::OpenAiCompatible,
        "gpt-4.1-mini",
        ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(SecretRef::new("env", "OPENAI_API_KEY").expect("valid secret ref")),
        true,
    )
    .expect("valid OpenAI-compatible provider")
}

fn runtime_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: default_provider.to_string(),
            catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
            invocation_policy: invocation_policy(),
        },
    }
}

fn effective_config(
    selected_provider: &str,
    selected_model: &str,
    providers: Vec<ModelProviderConfig>,
) -> EffectiveRuntimeConfig {
    EffectiveRuntimeConfig {
        config: runtime_config(selected_provider, providers),
        selected_provider: selected_provider.to_string(),
        selected_model: selected_model.to_string(),
        layers: vec![ConfigLayerSummary {
            kind: agent_core::ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            default_provider: selected_provider.to_string(),
            provider_ids: vec![selected_provider.to_string()],
        }],
        diagnostics: Vec::<ConfigDiagnostic>::new(),
    }
}

fn built_effective_config() -> EffectiveRuntimeConfig {
    let layer = agent_core::ConfigLayer {
        kind: agent_core::ConfigLayerKind::BuiltIn,
        source: "built-in".to_string(),
        config: runtime_config("fake", vec![fake_provider("fake", "fake-scripted", true)]),
    };
    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
}

fn registered_fake(id: &str, caps: ModelCapabilities) -> RegisteredModelProvider {
    RegisteredModelProvider::new(
        fake_provider(id, "fake-scripted", true),
        caps,
        Arc::new(FakeModelProvider::new(FakeModelScript::text("ready"))),
    )
    .expect("valid registered provider")
}

fn registry(providers: Vec<RegisteredModelProvider>) -> ModelProviderRegistry {
    ModelProviderRegistry::new(providers).expect("valid registry")
}

#[test]
fn model_runtime_resolver_selects_effective_provider() {
    let effective = built_effective_config();
    let registry = registry(vec![registered_fake(
        "fake",
        capabilities(true, false, false, Some(4096)),
    )]);

    let runtime = ModelRuntimeResolver::resolve(&effective, &registry, &prompt_requirements())
        .expect("resolved runtime");

    assert_eq!(runtime.provider_id, "fake");
    assert_eq!(runtime.model, "fake-scripted");
    assert_eq!(
        runtime.capabilities,
        capabilities(true, false, false, Some(4096))
    );
    assert_eq!(runtime.invocation_policy, invocation_policy());
}

#[test]
fn model_runtime_resolver_rejects_unregistered_provider() {
    let effective = built_effective_config();
    let registry = registry(Vec::new());

    let error = ModelRuntimeResolver::resolve(&effective, &registry, &prompt_requirements())
        .expect_err("missing provider rejected");

    assert!(matches!(
        error,
        ModelRuntimeError::ProviderNotRegistered { provider_id } if provider_id == "fake"
    ));
}

#[test]
fn model_runtime_resolver_rejects_disabled_provider() {
    let effective = effective_config(
        "openai",
        "gpt-4.1-mini",
        vec![
            fake_provider("fake", "fake-scripted", true),
            ModelProviderConfig {
                enabled: false,
                ..openai_provider()
            },
        ],
    );
    let registry = registry(vec![registered_fake(
        "openai",
        capabilities(true, false, false, Some(4096)),
    )]);

    let error = ModelRuntimeResolver::resolve(&effective, &registry, &prompt_requirements())
        .expect_err("disabled provider rejected");

    assert!(matches!(
        error,
        ModelRuntimeError::ProviderDisabled { provider_id } if provider_id == "openai"
    ));
}

#[test]
fn model_runtime_resolver_rejects_missing_capabilities() {
    let effective = built_effective_config();
    let registry = registry(vec![registered_fake(
        "fake",
        capabilities(true, false, false, Some(512)),
    )]);

    let error = ModelRuntimeResolver::resolve(&effective, &registry, &prompt_requirements())
        .expect_err("missing capability rejected");

    assert!(matches!(
        error,
        ModelRuntimeError::MissingCapabilities {
            provider_id,
            missing,
        } if provider_id == "fake" && missing == vec![ModelCapability::ContextTokens]
    ));
}

#[test]
fn model_runtime_resolver_error_output_excludes_secret_values() {
    let effective = effective_config("openai", "gpt-4.1-mini", vec![openai_provider()]);
    let registry = registry(Vec::new());

    let error = ModelRuntimeResolver::resolve(&effective, &registry, &prompt_requirements())
        .expect_err("missing provider rejected");
    let rendered = format!("{error:?} {error}");

    assert!(rendered.contains("openai"));
    assert!(!rendered.contains("OPENAI_API_KEY"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}
