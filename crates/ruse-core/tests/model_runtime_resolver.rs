use std::sync::Arc;

use ruse_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigDiagnostic, ConfigLayerSummary,
    ConfigOverride, EffectiveRuntimeConfig, ModelRuntimeError, ModelRuntimeResolver,
    ProviderSupportReport, ProviderSupportRole, RuntimeModelConfig, build_effective_runtime_config,
    coding_agent_model_requirements,
};
use ruse_model::{
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
    invocation_policy_with_fallback(Vec::new())
}

fn invocation_policy_with_fallback(fallback_provider_ids: Vec<&str>) -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
        ModelFallbackChain::new(fallback_provider_ids).expect("valid fallback"),
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

fn local_provider(id: &str, model: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Local,
        model,
        ModelEndpointConfig::new(Some("http://localhost:11434"), 30_000).expect("valid endpoint"),
        None,
        enabled,
    )
    .expect("valid local provider")
}

fn runtime_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> AgentRuntimeConfig {
    runtime_config_with_policy(default_provider, providers, invocation_policy())
}

fn runtime_config_with_policy(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
    invocation_policy: ModelInvocationPolicy,
) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: default_provider.to_string(),
            catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
            invocation_policy,
        },
        terminal: Default::default(),
    }
}

fn effective_config(
    selected_provider: &str,
    selected_model: &str,
    providers: Vec<ModelProviderConfig>,
) -> EffectiveRuntimeConfig {
    effective_config_with_policy(
        selected_provider,
        selected_model,
        providers,
        invocation_policy(),
    )
}

fn effective_config_with_policy(
    selected_provider: &str,
    selected_model: &str,
    providers: Vec<ModelProviderConfig>,
    invocation_policy: ModelInvocationPolicy,
) -> EffectiveRuntimeConfig {
    EffectiveRuntimeConfig {
        config: runtime_config_with_policy(selected_provider, providers, invocation_policy),
        selected_provider: selected_provider.to_string(),
        selected_model: selected_model.to_string(),
        layers: vec![ConfigLayerSummary {
            kind: ruse_core::ConfigLayerKind::BuiltIn,
            source: "test".to_string(),
            default_provider: selected_provider.to_string(),
            provider_ids: vec![selected_provider.to_string()],
        }],
        diagnostics: Vec::<ConfigDiagnostic>::new(),
    }
}

fn built_effective_config() -> EffectiveRuntimeConfig {
    let layer = ruse_core::ConfigLayer {
        kind: ruse_core::ConfigLayerKind::BuiltIn,
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

fn registered_config(
    config: ModelProviderConfig,
    caps: ModelCapabilities,
) -> RegisteredModelProvider {
    RegisteredModelProvider::new(
        config,
        caps,
        Arc::new(FakeModelProvider::new(FakeModelScript::text("ready"))),
    )
    .expect("valid registered provider")
}

fn registry(providers: Vec<RegisteredModelProvider>) -> ModelProviderRegistry {
    ModelProviderRegistry::new(providers).expect("valid registry")
}

#[test]
fn provider_support_report_marks_selected_and_fallback_compatibility() {
    let effective = effective_config_with_policy(
        "openai",
        "gpt-4.1-mini",
        vec![
            openai_provider(),
            fake_provider("fake", "fake-scripted", true),
            local_provider("local", "llama3.1", true),
        ],
        invocation_policy_with_fallback(vec!["fake", "local"]),
    );

    let report = ProviderSupportReport::from_effective_config(
        &effective,
        &coding_agent_model_requirements(),
    );

    let selected = report
        .providers
        .iter()
        .find(|provider| provider.provider_id == "openai")
        .expect("selected provider summary");
    assert_eq!(selected.role, ProviderSupportRole::Selected);
    assert!(selected.compatible);
    assert!(selected.missing.is_empty());

    let fake = report
        .providers
        .iter()
        .find(|provider| provider.provider_id == "fake")
        .expect("fake fallback summary");
    assert_eq!(fake.role, ProviderSupportRole::Fallback);
    assert_eq!(fake.fallback_index, Some(0));
    assert_eq!(fake.missing, vec![ModelCapability::ToolCalls]);

    let local = report
        .providers
        .iter()
        .find(|provider| provider.provider_id == "local")
        .expect("local fallback summary");
    assert_eq!(local.role, ProviderSupportRole::Fallback);
    assert_eq!(local.fallback_index, Some(1));
    assert!(local.compatible);
    assert!(local.missing.is_empty());

    let rendered = format!("{report:?}");
    assert!(!rendered.contains("OPENAI_API_KEY"));
    assert!(!rendered.contains("localhost:11434"));
    assert!(!rendered.contains("api_key"));
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
fn model_runtime_resolver_uses_compatible_fallback_when_selected_missing_capabilities() {
    let primary_config = fake_provider("primary", "primary-model", true);
    let tiny_config = fake_provider("tiny", "tiny-model", true);
    let capable_config = fake_provider("capable", "capable-model", true);
    let effective = effective_config_with_policy(
        "primary",
        "primary-model",
        vec![
            primary_config.clone(),
            tiny_config.clone(),
            capable_config.clone(),
        ],
        invocation_policy_with_fallback(vec!["tiny", "capable"]),
    );
    let registry = registry(vec![
        registered_config(primary_config, capabilities(true, false, false, Some(4096))),
        registered_config(tiny_config, capabilities(true, false, false, Some(4096))),
        registered_config(capable_config, capabilities(true, true, false, Some(4096))),
    ]);

    let runtime =
        ModelRuntimeResolver::resolve(&effective, &registry, &coding_agent_model_requirements())
            .expect("compatible fallback selected");

    assert_eq!(runtime.provider_id, "capable");
    assert_eq!(runtime.model, "capable-model");
    assert!(runtime.capabilities.tool_calls);
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
