use std::fmt;
use std::sync::Arc;

use agent_model::{
    ModelCapabilities, ModelCapability, ModelInvocationPolicy, ModelProvider,
    ModelProviderRegistry, ModelRegistryError, ModelRequestRequirements,
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
