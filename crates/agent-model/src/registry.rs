use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{ModelConfigError, ModelProviderConfig, ModelProviderKind};
use crate::provider::ModelProvider;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    #[serde(default)]
    pub image_input: bool,
    pub max_context_tokens: Option<u32>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub cost: Option<ModelCostProfile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelCostProfile {
    pub input_per_million: Option<String>,
    pub output_per_million: Option<String>,
    pub currency: String,
}

impl ModelCapabilities {
    #[must_use]
    pub fn fake_default() -> Self {
        Self {
            streaming: true,
            tool_calls: false,
            structured_outputs: false,
            image_input: false,
            max_context_tokens: Some(4096),
            max_output_tokens: Some(1024),
            cost: None,
        }
    }

    #[must_use]
    pub fn openai_compatible_default() -> Self {
        Self {
            streaming: true,
            tool_calls: false,
            structured_outputs: false,
            image_input: false,
            max_context_tokens: Some(128_000),
            max_output_tokens: Some(16_384),
            cost: None,
        }
    }

    #[must_use]
    pub fn anthropic_default() -> Self {
        Self {
            streaming: true,
            tool_calls: true,
            structured_outputs: false,
            image_input: false,
            max_context_tokens: Some(200_000),
            max_output_tokens: Some(8192),
            cost: None,
        }
    }

    #[must_use]
    pub fn default_for_provider_kind(kind: ModelProviderKind) -> Self {
        match kind {
            ModelProviderKind::Fake => Self::fake_default(),
            ModelProviderKind::OpenAiCompatible => Self::openai_compatible_default(),
            ModelProviderKind::Anthropic => Self::anthropic_default(),
        }
    }

    pub fn validate(&self) -> Result<(), ModelCapabilityProfileError> {
        if let Some(0) = self.max_context_tokens {
            return Err(ModelCapabilityProfileError::InvalidMaxContextTokens {
                max_context_tokens: 0,
            });
        }
        if let Some(0) = self.max_output_tokens {
            return Err(ModelCapabilityProfileError::InvalidMaxOutputTokens {
                max_output_tokens: 0,
            });
        }
        if let Some(cost) = &self.cost {
            cost.validate()?;
        }
        Ok(())
    }

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
        if requirements.image_input && !self.image_input {
            missing.push(ModelCapability::ImageInput);
        }
        if let Some(min_context_tokens) = requirements.min_context_tokens {
            match self.max_context_tokens {
                Some(max_context_tokens) if max_context_tokens >= min_context_tokens => {}
                _ => missing.push(ModelCapability::ContextTokens),
            }
        }
        if let Some(min_output_tokens) = requirements.min_output_tokens {
            match self.max_output_tokens {
                Some(max_output_tokens) if max_output_tokens >= min_output_tokens => {}
                _ => missing.push(ModelCapability::OutputTokens),
            }
        }
        missing
    }

    #[must_use]
    pub fn missing_labels_for(&self, requirements: &ModelRequestRequirements) -> Vec<&'static str> {
        self.missing_for(requirements)
            .into_iter()
            .map(ModelCapability::as_str)
            .collect()
    }

    #[must_use]
    pub fn satisfies(&self, requirements: &ModelRequestRequirements) -> bool {
        self.missing_for(requirements).is_empty()
    }
}

impl ModelCostProfile {
    pub fn validate(&self) -> Result<(), ModelCapabilityProfileError> {
        if self.currency.trim().is_empty() {
            return Err(ModelCapabilityProfileError::EmptyCostCurrency);
        }
        if self
            .input_per_million
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ModelCapabilityProfileError::EmptyInputCost);
        }
        if self
            .output_per_million
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ModelCapabilityProfileError::EmptyOutputCost);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Streaming,
    ToolCalls,
    StructuredOutputs,
    ImageInput,
    ContextTokens,
    OutputTokens,
}

impl ModelCapability {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::ToolCalls => "tool_calls",
            Self::StructuredOutputs => "structured_outputs",
            Self::ImageInput => "image_input",
            Self::ContextTokens => "context_tokens",
            Self::OutputTokens => "output_tokens",
        }
    }
}

impl fmt::Display for ModelCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRequestRequirements {
    pub streaming: bool,
    pub tool_calls: bool,
    pub structured_outputs: bool,
    #[serde(default)]
    pub image_input: bool,
    pub min_context_tokens: Option<u32>,
    #[serde(default)]
    pub min_output_tokens: Option<u32>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ModelCapabilityProfileError {
    #[error("model max context tokens must be greater than zero, got {max_context_tokens}")]
    InvalidMaxContextTokens { max_context_tokens: u32 },
    #[error("model max output tokens must be greater than zero, got {max_output_tokens}")]
    InvalidMaxOutputTokens { max_output_tokens: u32 },
    #[error("model cost profile currency must not be empty")]
    EmptyCostCurrency,
    #[error("model cost profile input_per_million must not be empty")]
    EmptyInputCost,
    #[error("model cost profile output_per_million must not be empty")]
    EmptyOutputCost,
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
        capabilities
            .validate()
            .map_err(|source| ModelRegistryError::InvalidCapabilities {
                provider_id: config.id.clone(),
                source,
            })?;
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
    #[error("invalid capabilities for model provider {provider_id}: {source}")]
    InvalidCapabilities {
        provider_id: String,
        source: ModelCapabilityProfileError,
    },
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
