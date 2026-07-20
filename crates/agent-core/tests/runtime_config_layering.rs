use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentConfigError, AgentRuntimeConfig, ConfigLayer,
    ConfigLayerKind, ConfigOverride, RuntimeModelConfig, RuntimeTerminalConfig,
    RuntimeTerminalTheme, build_effective_runtime_config,
};
use agent_model::{
    ModelEndpointConfig, ModelFallbackChain, ModelInvocationPolicy, ModelProviderCatalog,
    ModelProviderConfig, ModelProviderKind, ModelRetryPolicy, SecretRef,
};

fn runtime_config(
    default_provider: &str,
    providers: Vec<ModelProviderConfig>,
) -> AgentRuntimeConfig {
    AgentRuntimeConfig {
        schema_version: AGENT_CONFIG_SCHEMA_VERSION,
        model: RuntimeModelConfig {
            default_provider: default_provider.to_string(),
            catalog: ModelProviderCatalog::new(providers).expect("valid catalog"),
            invocation_policy: ModelInvocationPolicy::new(
                30_000,
                ModelRetryPolicy::new(1, Vec::new()).expect("valid retry"),
                ModelFallbackChain::new(Vec::<String>::new()).expect("valid fallback"),
            )
            .expect("valid invocation policy"),
        },
        terminal: RuntimeTerminalConfig::default(),
    }
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

#[test]
fn config_layering_higher_priority_terminal_theme_wins() {
    let mut built_in_config = runtime_config("fake", vec![fake_provider("fake", "fake", true)]);
    built_in_config.terminal.theme = RuntimeTerminalTheme::System;
    let mut user_config = runtime_config("fake", vec![fake_provider("fake", "fake-user", true)]);
    user_config.terminal.theme = RuntimeTerminalTheme::Dark;

    let effective = build_effective_runtime_config(
        vec![
            layer(ConfigLayerKind::User, "user", user_config),
            layer(ConfigLayerKind::BuiltIn, "built-in", built_in_config),
        ],
        ConfigOverride::default(),
    )
    .expect("effective config");

    assert_eq!(effective.config.terminal.theme, RuntimeTerminalTheme::Dark);
}

fn openai_provider(id: &str, model: &str, enabled: bool) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::OpenAiCompatible,
        model,
        ModelEndpointConfig::new(Some("https://api.openai.com/v1"), 30_000)
            .expect("valid endpoint"),
        Some(SecretRef::new("env", "OPENAI_API_KEY").expect("secret ref")),
        enabled,
    )
    .expect("valid OpenAI-compatible provider")
}

fn layer(kind: ConfigLayerKind, source: &str, config: AgentRuntimeConfig) -> ConfigLayer {
    ConfigLayer {
        kind,
        source: source.to_string(),
        config,
    }
}

#[test]
fn config_layering_higher_priority_default_provider_wins() {
    let built_in = layer(
        ConfigLayerKind::BuiltIn,
        "built-in",
        runtime_config("fake", vec![fake_provider("fake", "fake-scripted", true)]),
    );
    let user = layer(
        ConfigLayerKind::User,
        "user",
        runtime_config(
            "openai",
            vec![openai_provider("openai", "gpt-4.1-mini", true)],
        ),
    );

    let effective = build_effective_runtime_config(vec![user, built_in], ConfigOverride::default())
        .expect("effective config");

    assert_eq!(effective.selected_provider, "openai");
    assert_eq!(effective.selected_model, "gpt-4.1-mini");
    assert_eq!(effective.layers.len(), 2);
}

#[test]
fn config_layering_merges_provider_entries_by_id() {
    let built_in = layer(
        ConfigLayerKind::BuiltIn,
        "built-in",
        runtime_config("fake", vec![fake_provider("fake", "fake-scripted", true)]),
    );
    let workspace = layer(
        ConfigLayerKind::Workspace,
        "workspace",
        runtime_config(
            "fake",
            vec![
                fake_provider("fake", "fake-workspace", true),
                fake_provider("local", "local-model", true),
            ],
        ),
    );

    let effective =
        build_effective_runtime_config(vec![workspace, built_in], ConfigOverride::default())
            .expect("effective config");

    let providers = effective
        .config
        .model
        .catalog
        .providers
        .iter()
        .map(|provider| (provider.id.as_str(), provider.default_model.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        providers,
        vec![("fake", "fake-workspace"), ("local", "local-model")]
    );
}

#[test]
fn config_layering_explicit_provider_and_model_override_win() {
    let built_in = layer(
        ConfigLayerKind::BuiltIn,
        "built-in",
        runtime_config(
            "fake",
            vec![
                fake_provider("fake", "fake-scripted", true),
                openai_provider("openai", "gpt-4.1-mini", true),
            ],
        ),
    );

    let effective = build_effective_runtime_config(
        vec![built_in],
        ConfigOverride {
            provider: Some("openai".to_string()),
            model: Some("gpt-4.1".to_string()),
        },
    )
    .expect("effective config");

    assert_eq!(effective.selected_provider, "openai");
    assert_eq!(effective.selected_model, "gpt-4.1");
}

#[test]
fn config_layering_rejects_disabled_selected_provider() {
    let built_in = layer(
        ConfigLayerKind::BuiltIn,
        "built-in",
        runtime_config(
            "fake",
            vec![
                fake_provider("fake", "fake-scripted", true),
                openai_provider("openai", "gpt-4.1-mini", false),
            ],
        ),
    );

    let error = build_effective_runtime_config(
        vec![built_in],
        ConfigOverride {
            provider: Some("openai".to_string()),
            model: None,
        },
    )
    .expect_err("disabled provider rejected");

    let AgentConfigError::InvalidConfig { diagnostics } = error else {
        panic!("expected invalid config diagnostics");
    };
    assert_eq!(diagnostics[0].code, "config.selection.invalid");
    assert_eq!(
        diagnostics[0].field_path.as_deref(),
        Some("model.selected_provider")
    );
}

#[test]
fn config_layering_summary_serializes_without_secret_values() {
    let built_in = layer(
        ConfigLayerKind::BuiltIn,
        "built-in",
        runtime_config(
            "openai",
            vec![openai_provider("openai", "gpt-4.1-mini", true)],
        ),
    );

    let effective = build_effective_runtime_config(vec![built_in], ConfigOverride::default())
        .expect("effective config");
    let summary_json = serde_json::to_string(&effective.layers).expect("serialize layer summaries");

    assert!(summary_json.contains("openai"));
    assert!(!summary_json.contains("OPENAI_API_KEY"));
    assert!(!summary_json.contains("sk-test-secret"));
    assert!(!summary_json.contains("api_key_value"));
}
