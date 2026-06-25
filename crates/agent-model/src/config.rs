use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretRef {
    pub provider: String,
    pub name: String,
}

impl SecretRef {
    pub fn new(
        provider: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, ModelConfigError> {
        let secret_ref = Self {
            provider: provider.into(),
            name: name.into(),
        };
        secret_ref.validate()?;
        Ok(secret_ref)
    }

    pub fn validate(&self) -> Result<(), ModelConfigError> {
        let provider = self.provider.trim();
        if provider.is_empty() {
            return Err(ModelConfigError::EmptySecretProvider);
        }
        if !is_safe_identifier(provider) {
            return Err(ModelConfigError::InvalidSecretProvider {
                provider: self.provider.clone(),
            });
        }
        if self.name.trim().is_empty() {
            return Err(ModelConfigError::EmptySecretName);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProviderKind {
    Fake,
    OpenAiCompatible,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelEndpointConfig {
    pub base_url: Option<String>,
    pub timeout_ms: u64,
}

impl ModelEndpointConfig {
    pub fn new(base_url: Option<&str>, timeout_ms: u64) -> Result<Self, ModelConfigError> {
        let endpoint = Self {
            base_url: base_url.map(str::trim).map(ToOwned::to_owned),
            timeout_ms,
        };
        endpoint.validate()?;
        Ok(endpoint)
    }

    pub fn validate(&self) -> Result<(), ModelConfigError> {
        if self.timeout_ms == 0 {
            return Err(ModelConfigError::InvalidTimeout {
                timeout_ms: self.timeout_ms,
            });
        }
        if let Some(base_url) = &self.base_url
            && !is_safe_endpoint_url(base_url)
        {
            return Err(ModelConfigError::UnsafeEndpointUrl {
                url: base_url.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelProviderConfig {
    pub id: String,
    pub kind: ModelProviderKind,
    pub default_model: String,
    pub endpoint: ModelEndpointConfig,
    pub api_key: Option<SecretRef>,
    pub enabled: bool,
}

impl ModelProviderConfig {
    pub fn new(
        id: impl Into<String>,
        kind: ModelProviderKind,
        default_model: impl Into<String>,
        endpoint: ModelEndpointConfig,
        api_key: Option<SecretRef>,
        enabled: bool,
    ) -> Result<Self, ModelConfigError> {
        let config = Self {
            id: id.into(),
            kind,
            default_model: default_model.into(),
            endpoint,
            api_key,
            enabled,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ModelConfigError> {
        validate_provider_id(&self.id)?;
        if self.default_model.trim().is_empty() {
            return Err(ModelConfigError::EmptyDefaultModel {
                provider_id: self.id.clone(),
            });
        }
        self.endpoint.validate()?;
        if let Some(api_key) = &self.api_key {
            api_key.validate()?;
        }
        if self.kind == ModelProviderKind::OpenAiCompatible && self.api_key.is_none() {
            return Err(ModelConfigError::MissingSecretRef {
                provider_id: self.id.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelProviderCatalog {
    pub providers: Vec<ModelProviderConfig>,
}

impl ModelProviderCatalog {
    pub fn new(providers: Vec<ModelProviderConfig>) -> Result<Self, ModelConfigError> {
        let catalog = Self { providers };
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<(), ModelConfigError> {
        let mut ids = BTreeSet::new();
        let mut has_enabled_provider = false;
        for provider in &self.providers {
            provider.validate()?;
            if !ids.insert(provider.id.as_str()) {
                return Err(ModelConfigError::DuplicateProviderId {
                    id: provider.id.clone(),
                });
            }
            has_enabled_provider |= provider.enabled;
        }
        if !has_enabled_provider {
            return Err(ModelConfigError::NoEnabledProviders);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ModelConfigError {
    #[error("model provider id must not be empty")]
    EmptyProviderId,
    #[error("invalid model provider id: {id}")]
    InvalidProviderId { id: String },
    #[error("model provider {provider_id} default model must not be empty")]
    EmptyDefaultModel { provider_id: String },
    #[error("model endpoint timeout must be greater than zero, got {timeout_ms}")]
    InvalidTimeout { timeout_ms: u64 },
    #[error("unsafe model endpoint url: {url}")]
    UnsafeEndpointUrl { url: String },
    #[error("model provider {provider_id} requires a secret reference")]
    MissingSecretRef { provider_id: String },
    #[error("duplicate model provider id: {id}")]
    DuplicateProviderId { id: String },
    #[error("model provider catalog must include at least one enabled provider")]
    NoEnabledProviders,
    #[error("secret reference provider must not be empty")]
    EmptySecretProvider,
    #[error("invalid secret reference provider: {provider}")]
    InvalidSecretProvider { provider: String },
    #[error("secret reference name must not be empty")]
    EmptySecretName,
}

fn validate_provider_id(id: &str) -> Result<(), ModelConfigError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(ModelConfigError::EmptyProviderId);
    }
    if !is_safe_identifier(id) {
        return Err(ModelConfigError::InvalidProviderId { id: id.to_string() });
    }
    Ok(())
}

fn is_safe_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.')
        })
}

fn is_safe_endpoint_url(url: &str) -> bool {
    if url.starts_with("https://") {
        return true;
    }

    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    is_local_http_host(rest, "localhost") || is_local_http_host(rest, "127.0.0.1")
}

fn is_local_http_host(rest: &str, host: &str) -> bool {
    rest == host || rest.starts_with(&format!("{host}/")) || rest.starts_with(&format!("{host}:"))
}
