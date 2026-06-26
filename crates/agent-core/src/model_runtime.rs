use std::fmt;
use std::sync::Arc;

use agent_model::{
    ModelCapabilities, ModelCapability, ModelError, ModelFailureKind, ModelInvocationPolicy,
    ModelProvider, ModelProviderRegistry, ModelRegistryError, ModelRequest,
    ModelRequestRequirements, ModelStreamResult,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::EffectiveRuntimeConfig;

#[derive(Clone)]
pub struct ConfiguredModelRuntime {
    pub provider_id: String,
    pub model: String,
    pub provider: Arc<dyn ModelProvider>,
    pub capabilities: ModelCapabilities,
    pub invocation_policy: ModelInvocationPolicy,
}

impl fmt::Debug for ConfiguredModelRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfiguredModelRuntime")
            .field("provider_id", &self.provider_id)
            .field("model", &self.model)
            .field("capabilities", &self.capabilities)
            .field("invocation_policy", &self.invocation_policy)
            .finish_non_exhaustive()
    }
}

pub struct ModelRuntimeResolver;

impl ModelRuntimeResolver {
    pub fn resolve(
        effective: &EffectiveRuntimeConfig,
        registry: &ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
    ) -> Result<ConfiguredModelRuntime, ModelRuntimeError> {
        let selected_provider = effective.selected_provider.trim();
        if selected_provider.is_empty() {
            return Err(ModelRuntimeError::InvalidSelection {
                field: "model.selected_provider".to_string(),
                message: "selected provider must not be empty".to_string(),
            });
        }

        let selected_model = effective.selected_model.trim();
        if selected_model.is_empty() {
            return Err(ModelRuntimeError::InvalidSelection {
                field: "model.selected_model".to_string(),
                message: "selected model must not be empty".to_string(),
            });
        }

        let provider_config = effective
            .config
            .model
            .catalog
            .providers
            .iter()
            .find(|provider| provider.id == selected_provider)
            .ok_or_else(|| ModelRuntimeError::InvalidSelection {
                field: "model.selected_provider".to_string(),
                message: format!(
                    "selected provider {selected_provider} was not found in the effective catalog"
                ),
            })?;

        if !provider_config.enabled {
            return Err(ModelRuntimeError::ProviderDisabled {
                provider_id: selected_provider.to_string(),
            });
        }

        let registered = registry
            .require_capabilities(selected_provider, requirements)
            .map_err(ModelRuntimeError::from_registry)?;

        Ok(ConfiguredModelRuntime {
            provider_id: selected_provider.to_string(),
            model: selected_model.to_string(),
            provider: registered.provider(),
            capabilities: registered.capabilities.clone(),
            invocation_policy: effective.config.model.invocation_policy.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInvocationAttempt {
    pub provider_id: String,
    pub model: String,
    pub attempt_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInvocationOutcome {
    pub provider_id: String,
    pub model: String,
    pub attempts: Vec<ModelInvocationAttempt>,
    pub stream: ModelStreamResult,
}

pub struct ModelInvocationRunner;

impl ModelInvocationRunner {
    pub async fn run(
        runtime: &ConfiguredModelRuntime,
        registry: &ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
        request_template: ModelRequest,
    ) -> Result<ModelInvocationOutcome, ModelInvocationError> {
        let selected_provider = registry
            .get(&runtime.provider_id)
            .map_err(ModelInvocationError::selected_provider_unavailable)?;
        let use_model_override = runtime.model != selected_provider.config.default_model;
        let mut attempts = Vec::new();
        let mut next_attempt_index = 1;
        let mut last_failure = None;

        for _ in 0..runtime.invocation_policy.retry.max_attempts {
            let attempt = attempt(&runtime.provider_id, &runtime.model, next_attempt_index);
            next_attempt_index += 1;
            attempts.push(attempt);

            match invoke(
                Arc::clone(&runtime.provider),
                &runtime.provider_id,
                &runtime.model,
                &request_template,
            )
            .await
            {
                Ok(stream) => {
                    return Ok(ModelInvocationOutcome {
                        provider_id: runtime.provider_id.clone(),
                        model: runtime.model.clone(),
                        attempts,
                        stream,
                    });
                }
                Err(error) => {
                    let failure = InvocationFailure::new(
                        runtime.provider_id.clone(),
                        runtime.model.clone(),
                        error,
                    );
                    let should_retry = failure.retryable
                        && runtime
                            .invocation_policy
                            .retry
                            .is_retryable(failure.failure_kind)
                        && attempts.len() < runtime.invocation_policy.retry.max_attempts as usize;
                    last_failure = Some(failure);
                    if !should_retry {
                        break;
                    }
                }
            }
        }

        for fallback_provider_id in &runtime.invocation_policy.fallback.provider_ids {
            let registered = match registry.require_capabilities(fallback_provider_id, requirements)
            {
                Ok(registered) => registered,
                Err(ModelRegistryError::ProviderNotFound { id }) => {
                    return Err(ModelInvocationError::FallbackProviderNotRegistered {
                        provider_id: id,
                        attempts,
                    });
                }
                Err(ModelRegistryError::MissingCapabilities {
                    provider_id,
                    missing,
                }) => {
                    return Err(ModelInvocationError::FallbackMissingCapabilities {
                        provider_id,
                        missing,
                        attempts,
                    });
                }
                Err(error) => {
                    return Err(ModelInvocationError::SelectedProviderUnavailable {
                        provider_id: fallback_provider_id.clone(),
                        message: error.to_string(),
                        attempts,
                    });
                }
            };
            let fallback_model = if use_model_override {
                runtime.model.clone()
            } else {
                registered.config.default_model.clone()
            };
            let attempt = attempt(fallback_provider_id, &fallback_model, next_attempt_index);
            next_attempt_index += 1;
            attempts.push(attempt);

            match invoke(
                registered.provider(),
                fallback_provider_id,
                &fallback_model,
                &request_template,
            )
            .await
            {
                Ok(stream) => {
                    return Ok(ModelInvocationOutcome {
                        provider_id: fallback_provider_id.clone(),
                        model: fallback_model,
                        attempts,
                        stream,
                    });
                }
                Err(error) => {
                    last_failure = Some(InvocationFailure::new(
                        fallback_provider_id.clone(),
                        fallback_model,
                        error,
                    ));
                }
            }
        }

        let failure = last_failure.expect("at least one selected provider attempt was recorded");
        Err(ModelInvocationError::InvocationFailed {
            provider_id: failure.provider_id,
            model: failure.model,
            failure_kind: failure.failure_kind,
            retryable: failure.retryable,
            message: failure.message,
            attempts,
        })
    }
}

async fn invoke(
    provider: Arc<dyn ModelProvider>,
    provider_id: &str,
    model: &str,
    request_template: &ModelRequest,
) -> Result<ModelStreamResult, ModelError> {
    let mut request = request_template.clone();
    request.provider = provider_id.to_string();
    request.model = model.to_string();
    provider.stream(request).await
}

fn attempt(provider_id: &str, model: &str, attempt_index: u32) -> ModelInvocationAttempt {
    ModelInvocationAttempt {
        provider_id: provider_id.to_string(),
        model: model.to_string(),
        attempt_index,
    }
}

struct InvocationFailure {
    provider_id: String,
    model: String,
    failure_kind: ModelFailureKind,
    retryable: bool,
    message: String,
}

impl InvocationFailure {
    fn new(provider_id: String, model: String, error: ModelError) -> Self {
        let (failure_kind, retryable) = classify_model_error(&error);
        Self {
            provider_id,
            model,
            failure_kind,
            retryable,
            message: error.to_string(),
        }
    }
}

fn classify_model_error(error: &ModelError) -> (ModelFailureKind, bool) {
    match error {
        ModelError::Script(_) => (ModelFailureKind::Unknown, false),
        ModelError::Provider {
            code, retryable, ..
        } => (classify_provider_error_code(code), *retryable),
    }
}

fn classify_provider_error_code(code: &str) -> ModelFailureKind {
    let code = code.to_ascii_lowercase();
    if code.contains("timeout") {
        ModelFailureKind::Timeout
    } else if code.contains("rate") {
        ModelFailureKind::RateLimited
    } else if code.contains("transport") || code.contains("network") || code.contains("connection")
    {
        ModelFailureKind::Transport
    } else if code.contains("credential")
        || code.contains("auth")
        || code.contains("401")
        || code.contains("unauthorized")
    {
        ModelFailureKind::InvalidCredentials
    } else if code.contains("unsupported") {
        ModelFailureKind::UnsupportedRequest
    } else if code.contains("reject") || code.contains("provider_response") {
        ModelFailureKind::ProviderRejected
    } else {
        ModelFailureKind::Unknown
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModelRuntimeError {
    #[error("invalid model runtime selection at {field}: {message}")]
    InvalidSelection { field: String, message: String },
    #[error("model provider is disabled: {provider_id}")]
    ProviderDisabled { provider_id: String },
    #[error("model provider is not registered: {provider_id}")]
    ProviderNotRegistered { provider_id: String },
    #[error("model provider {provider_id} is missing capabilities: {missing:?}")]
    MissingCapabilities {
        provider_id: String,
        missing: Vec<ModelCapability>,
    },
}

impl ModelRuntimeError {
    fn from_registry(error: ModelRegistryError) -> Self {
        match error {
            ModelRegistryError::ProviderNotFound { id } => {
                Self::ProviderNotRegistered { provider_id: id }
            }
            ModelRegistryError::MissingCapabilities {
                provider_id,
                missing,
            } => Self::MissingCapabilities {
                provider_id,
                missing,
            },
            ModelRegistryError::DuplicateProviderId { id } => Self::InvalidSelection {
                field: "model.providers".to_string(),
                message: format!("duplicate registered provider id: {id}"),
            },
            ModelRegistryError::InvalidProviderConfig(error) => Self::InvalidSelection {
                field: "model.providers".to_string(),
                message: format!("invalid registered provider config: {error}"),
            },
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq, Serialize, Deserialize)]
pub enum ModelInvocationError {
    #[error("selected model provider {provider_id} is unavailable: {message}")]
    SelectedProviderUnavailable {
        provider_id: String,
        message: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("fallback model provider is not registered: {provider_id}")]
    FallbackProviderNotRegistered {
        provider_id: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("fallback model provider {provider_id} is missing capabilities: {missing:?}")]
    FallbackMissingCapabilities {
        provider_id: String,
        missing: Vec<ModelCapability>,
        attempts: Vec<ModelInvocationAttempt>,
    },
    #[error("model invocation failed for {provider_id}/{model}: {message}")]
    InvocationFailed {
        provider_id: String,
        model: String,
        failure_kind: ModelFailureKind,
        retryable: bool,
        message: String,
        attempts: Vec<ModelInvocationAttempt>,
    },
}

impl ModelInvocationError {
    fn selected_provider_unavailable(error: ModelRegistryError) -> Self {
        match error {
            ModelRegistryError::ProviderNotFound { id } => Self::SelectedProviderUnavailable {
                provider_id: id,
                message: "selected provider was not found in the registry".to_string(),
                attempts: Vec::new(),
            },
            error => Self::SelectedProviderUnavailable {
                provider_id: "selected".to_string(),
                message: error.to_string(),
                attempts: Vec::new(),
            },
        }
    }
}
