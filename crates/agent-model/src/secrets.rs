use std::collections::BTreeMap;
use std::fmt;

use thiserror::Error;

use crate::SecretRef;

#[derive(Clone, Eq, PartialEq)]
pub struct SecretValue {
    value: String,
}

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Result<Self, SecretResolutionError> {
        Self::from_parts("<direct>", "<direct>", value)
    }

    pub(crate) fn from_parts(
        provider: &str,
        name: &str,
        value: impl Into<String>,
    ) -> Result<Self, SecretResolutionError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SecretResolutionError::EmptySecretValue {
                provider: provider.to_string(),
                name: name.to_string(),
            });
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretValue(<redacted>)")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

pub trait SecretResolver: Send + Sync {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemorySecretResolver {
    secrets: BTreeMap<(String, String), SecretValue>,
}

impl InMemorySecretResolver {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_secret(
        mut self,
        secret_ref: SecretRef,
        value: impl Into<String>,
    ) -> Result<Self, SecretResolutionError> {
        self.insert_secret(secret_ref, value)?;
        Ok(self)
    }

    pub fn insert_secret(
        &mut self,
        secret_ref: SecretRef,
        value: impl Into<String>,
    ) -> Result<(), SecretResolutionError> {
        let value = SecretValue::from_parts(&secret_ref.provider, &secret_ref.name, value)?;
        self.secrets.insert(secret_key(&secret_ref), value);
        Ok(())
    }
}

impl SecretResolver for InMemorySecretResolver {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        self.secrets
            .get(&secret_key(secret_ref))
            .cloned()
            .ok_or_else(|| SecretResolutionError::SecretNotFound {
                provider: secret_ref.provider.clone(),
                name: secret_ref.name.clone(),
            })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EnvSecretResolver;

impl SecretResolver for EnvSecretResolver {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        if secret_ref.provider != "env" {
            return Err(SecretResolutionError::UnsupportedSecretProvider {
                provider: secret_ref.provider.clone(),
            });
        }

        let value =
            std::env::var(&secret_ref.name).map_err(|_| SecretResolutionError::SecretNotFound {
                provider: secret_ref.provider.clone(),
                name: secret_ref.name.clone(),
            })?;
        SecretValue::from_parts(&secret_ref.provider, &secret_ref.name, value)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SecretResolutionError {
    #[error("secret value for {provider}/{name} must not be empty")]
    EmptySecretValue { provider: String, name: String },
    #[error("unsupported secret provider: {provider}")]
    UnsupportedSecretProvider { provider: String },
    #[error("secret not found: {provider}/{name}")]
    SecretNotFound { provider: String, name: String },
}

fn secret_key(secret_ref: &SecretRef) -> (String, String) {
    (secret_ref.provider.clone(), secret_ref.name.clone())
}
