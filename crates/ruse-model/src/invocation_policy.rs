use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::registry::{
    ModelProviderRegistry, ModelRegistryError, ModelRequestRequirements, RegisteredModelProvider,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelInvocationPolicy {
    pub timeout_ms: u64,
    pub retry: ModelRetryPolicy,
    pub fallback: ModelFallbackChain,
}

impl ModelInvocationPolicy {
    pub fn new(
        timeout_ms: u64,
        retry: ModelRetryPolicy,
        fallback: ModelFallbackChain,
    ) -> Result<Self, ModelInvocationPolicyError> {
        let policy = Self {
            timeout_ms,
            retry,
            fallback,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> Result<(), ModelInvocationPolicyError> {
        if self.timeout_ms == 0 {
            return Err(ModelInvocationPolicyError::InvalidTimeout {
                timeout_ms: self.timeout_ms,
            });
        }
        self.retry.validate()?;
        self.fallback.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelRetryPolicy {
    pub max_attempts: u32,
    pub retryable_failures: Vec<ModelFailureKind>,
}

impl ModelRetryPolicy {
    pub fn new(
        max_attempts: u32,
        retryable_failures: Vec<ModelFailureKind>,
    ) -> Result<Self, ModelInvocationPolicyError> {
        let policy = Self {
            max_attempts,
            retryable_failures,
        };
        policy.validate()?;
        Ok(policy)
    }

    pub fn validate(&self) -> Result<(), ModelInvocationPolicyError> {
        if self.max_attempts == 0 {
            return Err(ModelInvocationPolicyError::InvalidRetryAttempts {
                max_attempts: self.max_attempts,
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn is_retryable(&self, failure: ModelFailureKind) -> bool {
        self.retryable_failures.contains(&failure)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFailureKind {
    Timeout,
    RateLimited,
    Transport,
    InvalidCredentials,
    UnsupportedRequest,
    ProviderRejected,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelFallbackChain {
    pub provider_ids: Vec<String>,
}

impl ModelFallbackChain {
    pub fn new<I, S>(provider_ids: I) -> Result<Self, ModelInvocationPolicyError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let chain = Self {
            provider_ids: provider_ids
                .into_iter()
                .map(Into::into)
                .map(|id| id.trim().to_string())
                .collect(),
        };
        chain.validate()?;
        Ok(chain)
    }

    pub fn validate(&self) -> Result<(), ModelInvocationPolicyError> {
        let mut seen = BTreeSet::new();
        for id in &self.provider_ids {
            if id.trim().is_empty() {
                return Err(ModelInvocationPolicyError::InvalidFallbackProviderId {
                    id: id.clone(),
                });
            }
            if !seen.insert(id.as_str()) {
                return Err(ModelInvocationPolicyError::DuplicateFallbackProviderId {
                    id: id.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn select_first_compatible<'registry>(
        &self,
        registry: &'registry ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
    ) -> Result<Option<&'registry RegisteredModelProvider>, ModelInvocationPolicyError> {
        for provider_id in &self.provider_ids {
            let provider = registry.get(provider_id)?;
            if provider.capabilities.satisfies(requirements) {
                return Ok(Some(provider));
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ModelInvocationPolicyError {
    #[error("model invocation timeout must be greater than zero, got {timeout_ms}")]
    InvalidTimeout { timeout_ms: u64 },
    #[error("model retry max attempts must be greater than zero, got {max_attempts}")]
    InvalidRetryAttempts { max_attempts: u32 },
    #[error("invalid fallback provider id: {id}")]
    InvalidFallbackProviderId { id: String },
    #[error("duplicate fallback provider id: {id}")]
    DuplicateFallbackProviderId { id: String },
    #[error(transparent)]
    Registry(#[from] ModelRegistryError),
}
