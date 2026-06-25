use std::sync::Arc;

use agent_model::{
    FakeModelProvider, FakeModelScript, ModelCapabilities, ModelEndpointConfig, ModelFailureKind,
    ModelFallbackChain, ModelInvocationPolicy, ModelInvocationPolicyError, ModelProviderConfig,
    ModelProviderKind, ModelProviderRegistry, ModelRequestRequirements, ModelRetryPolicy,
    RegisteredModelProvider,
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

fn registered(id: &str, capabilities: ModelCapabilities) -> RegisteredModelProvider {
    RegisteredModelProvider::new(
        fake_config(id),
        capabilities,
        Arc::new(FakeModelProvider::new(FakeModelScript::text("ready"))),
    )
    .expect("valid registered provider")
}

#[test]
fn invocation_policy_serializes_stable_shape() {
    let policy = ModelInvocationPolicy::new(
        30_000,
        ModelRetryPolicy::new(
            3,
            vec![
                ModelFailureKind::Timeout,
                ModelFailureKind::RateLimited,
                ModelFailureKind::Transport,
            ],
        )
        .expect("valid retry policy"),
        ModelFallbackChain::new(vec!["openai", "local"]).expect("valid fallback chain"),
    )
    .expect("valid invocation policy");

    assert_eq!(
        serde_json::to_value(policy).expect("serialize policy"),
        json!({
            "timeout_ms": 30000,
            "retry": {
                "max_attempts": 3,
                "retryable_failures": ["timeout", "rate_limited", "transport"]
            },
            "fallback": {
                "provider_ids": ["openai", "local"]
            }
        })
    );
}

#[test]
fn invocation_policy_rejects_zero_timeout() {
    let retry =
        ModelRetryPolicy::new(2, vec![ModelFailureKind::Timeout]).expect("valid retry policy");
    let fallback = ModelFallbackChain::new(Vec::<String>::new()).expect("empty fallback allowed");

    let timeout_err =
        ModelInvocationPolicy::new(0, retry, fallback).expect_err("zero timeout rejected");
    let attempts_err = ModelRetryPolicy::new(0, vec![ModelFailureKind::Timeout])
        .expect_err("zero attempts rejected");

    assert!(matches!(
        timeout_err,
        ModelInvocationPolicyError::InvalidTimeout { timeout_ms: 0 }
    ));
    assert!(matches!(
        attempts_err,
        ModelInvocationPolicyError::InvalidRetryAttempts { max_attempts: 0 }
    ));
}

#[test]
fn retry_policy_classifies_retryable_errors() {
    let retry = ModelRetryPolicy::new(
        3,
        vec![
            ModelFailureKind::Timeout,
            ModelFailureKind::RateLimited,
            ModelFailureKind::Transport,
        ],
    )
    .expect("valid retry policy");

    assert!(retry.is_retryable(ModelFailureKind::Timeout));
    assert!(retry.is_retryable(ModelFailureKind::RateLimited));
    assert!(retry.is_retryable(ModelFailureKind::Transport));
    assert!(!retry.is_retryable(ModelFailureKind::InvalidCredentials));
    assert!(!retry.is_retryable(ModelFailureKind::UnsupportedRequest));
}

#[test]
fn fallback_chain_rejects_empty_and_duplicate_provider_ids() {
    let empty_err =
        ModelFallbackChain::new(vec!["primary", " "]).expect_err("empty provider id rejected");
    let duplicate_err = ModelFallbackChain::new(vec!["primary", "secondary", "primary"])
        .expect_err("duplicate provider id rejected");

    assert!(matches!(
        empty_err,
        ModelInvocationPolicyError::InvalidFallbackProviderId { id } if id.trim().is_empty()
    ));
    assert!(matches!(
        duplicate_err,
        ModelInvocationPolicyError::DuplicateFallbackProviderId { id } if id == "primary"
    ));
}

#[test]
fn fallback_chain_selects_first_compatible_provider() {
    let registry = ModelProviderRegistry::new(vec![
        registered(
            "tiny",
            ModelCapabilities {
                streaming: true,
                tool_calls: false,
                structured_outputs: false,
                max_context_tokens: Some(1024),
            },
        ),
        registered(
            "capable",
            ModelCapabilities {
                streaming: true,
                tool_calls: true,
                structured_outputs: false,
                max_context_tokens: Some(8192),
            },
        ),
    ])
    .expect("valid registry");
    let fallback = ModelFallbackChain::new(vec!["tiny", "capable"]).expect("valid fallback chain");
    let requirements = ModelRequestRequirements {
        streaming: true,
        tool_calls: true,
        structured_outputs: false,
        min_context_tokens: Some(4096),
    };

    let selected = fallback
        .select_first_compatible(&registry, &requirements)
        .expect("fallback selection")
        .expect("compatible fallback");

    assert_eq!(selected.config.id, "capable");
}
