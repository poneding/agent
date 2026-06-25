use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{ModelConfigError, ModelProviderConfig};
use crate::provider::ModelProvider;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    pub max_context_tokens: Option<u32>,
}

impl ModelCapabilities {
    #[must_use]
    pub fn missing_for(&self, requirements: &ModelRequestRequirements) -> Vec<ModelCapability> {
        let mut missing = Vec::new();
        if requirements.streaming && !self.streaming {
            missing.push(ModelCapability::Streaming);
        }
        if requirements.tool_calls && !self.tool_calls {
            missing.push(ModelCapability::ToolCalls);
        }
        if requirements.structured_outputs && !self.structured_outputs {
            missing.push(ModelCapability::StructuredOutputs);
        }
        if let Some(min_context_tokens) = requirements.min_context_tokens {
            match self.max_context_tokens {
                Some(max_context_tokens) if max_context_tokens >= min_context_tokens => {}
                _ => missing.push(ModelCapability::ContextTokens),
            }
        }
        missing
    }

    #[must_use]
    pub fn satisfies(&self, requirements: &ModelRequestRequirements) -> bool {
        self.missing_for(requirements).is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Streaming,
    ToolCalls,
    StructuredOutputs,
    ContextTokens,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRequestRequirements {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    pub min_context_tokens: Option<u32>,
}

#[derive(Clone)]
pub struct RegisteredModelProvider {
    pub config: ModelProviderConfig,
    pub capabilities: ModelCapabilities,
    provider: Arc<dyn ModelProvider>,
}

impl RegisteredModelProvider {
    pub fn new(
        config: ModelProviderConfig,
        capabilities: ModelCapabilities,
        provider: Arc<dyn ModelProvider>,
    ) -> Result<Self, ModelRegistryError> {
        config
            .validate()
            .map_err(ModelRegistryError::InvalidProviderConfig)?;
        Ok(Self {
            config,
            capabilities,
            provider,
        })
    }

    #[must_use]
    pub fn provider(&self) -> Arc<dyn ModelProvider> {
        Arc::clone(&self.provider)
    }
}

impl fmt::Debug for RegisteredModelProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredModelProvider")
            .field("config", &self.config)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct ModelProviderRegistry {
    providers: BTreeMap<String, RegisteredModelProvider>,
}

impl ModelProviderRegistry {
    pub fn new(providers: Vec<RegisteredModelProvider>) -> Result<Self, ModelRegistryError> {
        let mut by_id = BTreeMap::new();
        for provider in providers {
            let id = provider.config.id.clone();
            if by_id.contains_key(&id) {
                return Err(ModelRegistryError::DuplicateProviderId { id });
            }
            by_id.insert(id, provider);
        }
        Ok(Self { providers: by_id })
    }

    pub fn get(&self, id: &str) -> Result<&RegisteredModelProvider, ModelRegistryError> {
        let id = id.trim();
        self.providers
            .get(id)
            .ok_or_else(|| ModelRegistryError::ProviderNotFound { id: id.to_string() })
    }

    pub fn require_capabilities(
        &self,
        id: &str,
        requirements: &ModelRequestRequirements,
    ) -> Result<&RegisteredModelProvider, ModelRegistryError> {
        let provider = self.get(id)?;
        let missing = provider.capabilities.missing_for(requirements);
        if missing.is_empty() {
            Ok(provider)
        } else {
            Err(ModelRegistryError::MissingCapabilities {
                provider_id: provider.config.id.clone(),
                missing,
            })
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ModelRegistryError {
    #[error("invalid provider config: {0}")]
    InvalidProviderConfig(ModelConfigError),
    #[error("duplicate model provider id: {id}")]
    DuplicateProviderId { id: String },
    #[error("model provider not found: {id}")]
    ProviderNotFound { id: String },
    #[error("model provider {provider_id} is missing capabilities: {missing:?}")]
    MissingCapabilities {
        provider_id: String,
        missing: Vec<ModelCapability>,
    },
}
