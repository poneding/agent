use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use agent_core::{
    AGENT_CONFIG_SCHEMA_VERSION, AgentRuntimeConfig, ConfigLayer, ConfigLayerKind, ConfigOverride,
    EffectiveRuntimeConfig, ModelInvocationError, ModelInvocationRunner, ModelRuntimeResolver,
    RuntimeModelConfig, build_effective_runtime_config,
};
use agent_model::{
    ModelCapabilities, ModelEndpointConfig, ModelError, ModelFailureKind, ModelFallbackChain,
    ModelInvocationPolicy, ModelProvider, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequest, ModelRequestRequirements,
    ModelRetryPolicy, ModelStreamEvent, ModelStreamResult, RegisteredModelProvider,
};
use agent_types::{ConversationId, CorrelationId, TurnId};
use async_trait::async_trait;

fn capabilities() -> ModelCapabilities {
    ModelCapabilities {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        max_context_tokens: Some(4096),
    }
}

fn requirements() -> ModelRequestRequirements {
    ModelRequestRequirements {
        streaming: true,
        tool_calls: false,
        structured_outputs: false,
        min_context_tokens: Some(1024),
    }
}

fn provider_config(id: &str, model: &str) -> ModelProviderConfig {
    ModelProviderConfig::new(
        id,
        ModelProviderKind::Fake,
        model,
        ModelEndpointConfig::new(None, 30_000).expect("valid endpoint"),
        None,
        true,
    )
    .expect("valid provider config")
}

fn invocation_policy(
    max_attempts: u32,
    retryable_failures: Vec<ModelFailureKind>,
    fallback_provider_ids: Vec<&str>,
) -> ModelInvocationPolicy {
    ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(max_attempts, retryable_failures).expect("valid retry policy"),
        ModelFallbackChain::new(fallback_provider_ids).expect("valid fallback chain"),
    )
    .expect("valid invocation policy")
}

fn runtime_config(
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
    }
}

fn effective_config(
    selected_provider: &str,
    providers: Vec<ModelProviderConfig>,
    invocation_policy: ModelInvocationPolicy,
) -> EffectiveRuntimeConfig {
    let layer = ConfigLayer {
        kind: ConfigLayerKind::BuiltIn,
        source: "test".to_string(),
        config: runtime_config(selected_provider, providers, invocation_policy),
    };
    build_effective_runtime_config(vec![layer], ConfigOverride::default())
        .expect("effective config")
}

fn registered(
    config: ModelProviderConfig,
    provider: Arc<dyn ModelProvider>,
) -> RegisteredModelProvider {
    RegisteredModelProvider::new(config, capabilities(), provider).expect("registered provider")
}

fn request() -> ModelRequest {
    ModelRequest {
        provider: "template-provider".to_string(),
        model: "template-model".to_string(),
        conversation_id: ConversationId::new(),
        turn_id: TurnId::new(),
        correlation_id: CorrelationId::new(),
        messages: vec!["hello with sk-test-secret should stay out of errors".to_string()],
    }
}

fn stream_text(text: &str) -> ModelStreamResult {
    vec![
        ModelStreamEvent::Started {
            request_id: format!("request-{text}"),
        },
        ModelStreamEvent::Delta {
            text: text.to_string(),
        },
        ModelStreamEvent::Completed {
            finish_reason: "stop".to_string(),
            output_token_count: text.len() as u32,
        },
    ]
}

fn provider_error(code: &str, retryable: bool) -> ModelError {
    ModelError::Provider {
        code: code.to_string(),
        message: "redacted provider failure".to_string(),
        retryable,
    }
}

struct RecordingProvider {
    outcomes: Mutex<VecDeque<Result<ModelStreamResult, ModelError>>>,
    requests: Arc<Mutex<Vec<ModelRequest>>>,
}

impl RecordingProvider {
    fn scripted(
        outcomes: Vec<Result<ModelStreamResult, ModelError>>,
    ) -> (Arc<Self>, Arc<Mutex<Vec<ModelRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let provider = Arc::new(Self {
            outcomes: Mutex::new(outcomes.into()),
            requests: Arc::clone(&requests),
        });
        (provider, requests)
    }
}

#[async_trait]
impl ModelProvider for RecordingProvider {
    async fn stream(&self, request: ModelRequest) -> Result<ModelStreamResult, ModelError> {
        self.requests.lock().expect("requests lock").push(request);
        self.outcomes
            .lock()
            .expect("outcomes lock")
            .pop_front()
            .unwrap_or_else(|| Err(ModelError::Script("script exhausted".to_string())))
    }
}

#[tokio::test]
async fn model_invocation_policy_retries_retryable_failure() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(2, vec![ModelFailureKind::Timeout], Vec::new()),
    );
    let (primary, primary_requests) = RecordingProvider::scripted(vec![
        Err(provider_error("model.timeout", true)),
        Ok(stream_text("ready")),
    ]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");
    let runtime =
        ModelRuntimeResolver::resolve(&effective, &registry, &requirements()).expect("runtime");

    let outcome = ModelInvocationRunner::run(&runtime, &registry, &requirements(), request())
        .await
        .expect("invocation succeeds after retry");

    assert_eq!(outcome.provider_id, "primary");
    assert_eq!(outcome.model, "primary-model");
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(outcome.attempts[0].attempt_index, 1);
    assert_eq!(outcome.attempts[1].attempt_index, 2);
    assert_eq!(outcome.stream, stream_text("ready"));
    assert_eq!(primary_requests.lock().expect("requests").len(), 2);
}

#[tokio::test]
async fn model_invocation_policy_does_not_retry_non_retryable_failure() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(3, vec![ModelFailureKind::InvalidCredentials], Vec::new()),
    );
    let (primary, primary_requests) = RecordingProvider::scripted(vec![Err(provider_error(
        "model.invalid_credentials",
        false,
    ))]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");
    let runtime =
        ModelRuntimeResolver::resolve(&effective, &registry, &requirements()).expect("runtime");

    let error = ModelInvocationRunner::run(&runtime, &registry, &requirements(), request())
        .await
        .expect_err("non-retryable failure stops immediately");

    assert!(matches!(
        error,
        ModelInvocationError::InvocationFailed {
            failure_kind: ModelFailureKind::InvalidCredentials,
            attempts,
            ..
        } if attempts.len() == 1
    ));
    assert_eq!(primary_requests.lock().expect("requests").len(), 1);
}

#[tokio::test]
async fn model_invocation_policy_uses_fallback_provider() {
    let primary_config = provider_config("primary", "primary-model");
    let backup_config = provider_config("backup", "backup-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone(), backup_config.clone()],
        invocation_policy(1, vec![ModelFailureKind::Transport], vec!["backup"]),
    );
    let (primary, primary_requests) =
        RecordingProvider::scripted(vec![Err(provider_error("model.transport", true))]);
    let (backup, backup_requests) = RecordingProvider::scripted(vec![Ok(stream_text("fallback"))]);
    let registry = ModelProviderRegistry::new(vec![
        registered(primary_config, primary),
        registered(backup_config, backup),
    ])
    .expect("registry");
    let runtime =
        ModelRuntimeResolver::resolve(&effective, &registry, &requirements()).expect("runtime");

    let outcome = ModelInvocationRunner::run(&runtime, &registry, &requirements(), request())
        .await
        .expect("fallback succeeds");

    assert_eq!(outcome.provider_id, "backup");
    assert_eq!(outcome.model, "backup-model");
    assert_eq!(outcome.attempts.len(), 2);
    assert_eq!(outcome.attempts[0].provider_id, "primary");
    assert_eq!(outcome.attempts[1].provider_id, "backup");
    assert_eq!(primary_requests.lock().expect("requests").len(), 1);
    let backup_requests = backup_requests.lock().expect("requests");
    assert_eq!(backup_requests.len(), 1);
    assert_eq!(backup_requests[0].provider, "backup");
    assert_eq!(backup_requests[0].model, "backup-model");
}

#[tokio::test]
async fn model_invocation_policy_reports_missing_fallback_safely() {
    let primary_config = provider_config("primary", "primary-model");
    let effective = effective_config(
        "primary",
        vec![primary_config.clone()],
        invocation_policy(1, vec![ModelFailureKind::Transport], vec!["missing"]),
    );
    let (primary, _primary_requests) =
        RecordingProvider::scripted(vec![Err(provider_error("model.transport", true))]);
    let registry =
        ModelProviderRegistry::new(vec![registered(primary_config, primary)]).expect("registry");
    let runtime =
        ModelRuntimeResolver::resolve(&effective, &registry, &requirements()).expect("runtime");

    let error = ModelInvocationRunner::run(&runtime, &registry, &requirements(), request())
        .await
        .expect_err("missing fallback rejected");
    let rendered = format!("{error:?} {error}");

    assert!(matches!(
        error,
        ModelInvocationError::FallbackProviderNotRegistered { provider_id, attempts }
            if provider_id == "missing" && attempts.len() == 1
    ));
    assert!(rendered.contains("missing"));
    assert!(!rendered.contains("sk-test-secret"));
    assert!(!rendered.contains("api_key"));
    assert!(!rendered.contains("token"));
    assert!(!rendered.contains("password"));
}
