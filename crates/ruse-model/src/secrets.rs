use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ModelConfigError, SecretRef};

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

#[derive(Clone, Default)]
pub struct CompositeSecretResolver {
    resolvers: Vec<Arc<dyn SecretResolver>>,
}

impl CompositeSecretResolver {
    #[must_use]
    pub fn new(resolvers: Vec<Arc<dyn SecretResolver>>) -> Self {
        Self { resolvers }
    }

    #[must_use]
    pub fn with_resolver(mut self, resolver: Arc<dyn SecretResolver>) -> Self {
        self.resolvers.push(resolver);
        self
    }
}

impl fmt::Debug for CompositeSecretResolver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompositeSecretResolver")
            .field("resolver_count", &self.resolvers.len())
            .finish()
    }
}

impl SecretResolver for CompositeSecretResolver {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        let mut last_error = None;
        for resolver in &self.resolvers {
            match resolver.resolve(secret_ref) {
                Ok(value) => return Ok(value),
                Err(error) => last_error = Some(error),
            }
        }

        Err(
            last_error.unwrap_or_else(|| SecretResolutionError::SecretNotFound {
                provider: secret_ref.provider.clone(),
                name: secret_ref.name.clone(),
            }),
        )
    }
}

pub trait SecretStoreBackend: Send + Sync {
    fn put(&mut self, name: &str, value: SecretValue) -> Result<SecretRef, SecretStoreError>;
    fn get(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError>;
    fn delete(&mut self, secret_ref: &SecretRef) -> Result<(), SecretStoreError>;
    fn health(&self) -> SecretStoreHealth;
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretStoreKind {
    OsKeychain,
    Environment,
    Memory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretStoreDiagnostic {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl SecretStoreDiagnostic {
    #[must_use]
    pub fn recoverable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretStoreHealth {
    pub kind: SecretStoreKind,
    pub available: bool,
    pub writable: bool,
    pub diagnostics: Vec<SecretStoreDiagnostic>,
}

impl SecretStoreHealth {
    #[must_use]
    pub fn healthy(kind: SecretStoreKind, writable: bool) -> Self {
        Self {
            kind,
            available: true,
            writable,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn unavailable(kind: SecretStoreKind, diagnostics: Vec<SecretStoreDiagnostic>) -> Self {
        Self {
            kind,
            available: false,
            writable: false,
            diagnostics,
        }
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SecretStoreError {
    #[error("invalid secret reference: {0}")]
    InvalidReference(String),
    #[error("secret store {kind:?} is read-only")]
    ReadOnly { kind: SecretStoreKind },
    #[error("secret store {kind:?} is unavailable: {message}")]
    Unavailable {
        kind: SecretStoreKind,
        message: String,
    },
    #[error("secret not found: {provider}/{name}")]
    SecretNotFound { provider: String, name: String },
    #[error("secret value for {provider}/{name} must not be empty")]
    EmptySecretValue { provider: String, name: String },
}

impl From<SecretResolutionError> for SecretStoreError {
    fn from(error: SecretResolutionError) -> Self {
        match error {
            SecretResolutionError::EmptySecretValue { provider, name } => {
                Self::EmptySecretValue { provider, name }
            }
            SecretResolutionError::UnsupportedSecretProvider { provider } => Self::Unavailable {
                kind: SecretStoreKind::Environment,
                message: format!("unsupported secret provider: {provider}"),
            },
            SecretResolutionError::SecretNotFound { provider, name } => {
                Self::SecretNotFound { provider, name }
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct InMemorySecretStore {
    provider: String,
    secrets: BTreeMap<(String, String), SecretValue>,
}

impl fmt::Debug for InMemorySecretStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InMemorySecretStore")
            .field("provider", &self.provider)
            .field("secret_count", &self.secrets.len())
            .finish()
    }
}

impl InMemorySecretStore {
    pub fn new(provider: impl Into<String>) -> Result<Self, SecretStoreError> {
        let provider = provider.into();
        SecretRef::new(&provider, "health-check").map_err(secret_ref_error)?;
        Ok(Self {
            provider,
            secrets: BTreeMap::new(),
        })
    }
}

impl SecretStoreBackend for InMemorySecretStore {
    fn put(&mut self, name: &str, value: SecretValue) -> Result<SecretRef, SecretStoreError> {
        let secret_ref = SecretRef::new(&self.provider, name).map_err(secret_ref_error)?;
        self.secrets.insert(secret_key(&secret_ref), value);
        Ok(secret_ref)
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        self.secrets
            .get(&secret_key(secret_ref))
            .cloned()
            .ok_or_else(|| SecretStoreError::SecretNotFound {
                provider: secret_ref.provider.clone(),
                name: secret_ref.name.clone(),
            })
    }

    fn delete(&mut self, secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
        self.secrets.remove(&secret_key(secret_ref));
        Ok(())
    }

    fn health(&self) -> SecretStoreHealth {
        SecretStoreHealth::healthy(SecretStoreKind::Memory, true)
    }
}

impl SecretResolver for InMemorySecretStore {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        self.get(secret_ref)
            .map_err(secret_store_to_resolution_error)
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct EnvSecretStore;

impl fmt::Debug for EnvSecretStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EnvSecretStore(<redacted>)")
    }
}

impl SecretStoreBackend for EnvSecretStore {
    fn put(&mut self, _name: &str, _value: SecretValue) -> Result<SecretRef, SecretStoreError> {
        Err(SecretStoreError::ReadOnly {
            kind: SecretStoreKind::Environment,
        })
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        EnvSecretResolver.resolve(secret_ref).map_err(Into::into)
    }

    fn delete(&mut self, _secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
        Err(SecretStoreError::ReadOnly {
            kind: SecretStoreKind::Environment,
        })
    }

    fn health(&self) -> SecretStoreHealth {
        SecretStoreHealth::healthy(SecretStoreKind::Environment, false)
    }
}

impl SecretResolver for EnvSecretStore {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        self.get(secret_ref)
            .map_err(secret_store_to_resolution_error)
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub struct OsKeychainSecretStore {
    env_fallback: EnvSecretStore,
}

impl fmt::Debug for OsKeychainSecretStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OsKeychainSecretStore(<redacted>)")
    }
}

impl OsKeychainSecretStore {
    #[must_use]
    pub fn unavailable_with_env_fallback() -> Self {
        Self {
            env_fallback: EnvSecretStore,
        }
    }
}

impl SecretStoreBackend for OsKeychainSecretStore {
    fn put(&mut self, _name: &str, _value: SecretValue) -> Result<SecretRef, SecretStoreError> {
        Err(SecretStoreError::Unavailable {
            kind: SecretStoreKind::OsKeychain,
            message: "OS keychain backend is not available in this build".to_string(),
        })
    }

    fn get(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        if secret_ref.provider == "env" {
            self.env_fallback.get(secret_ref)
        } else {
            Err(SecretStoreError::Unavailable {
                kind: SecretStoreKind::OsKeychain,
                message: "OS keychain backend is not available in this build".to_string(),
            })
        }
    }

    fn delete(&mut self, _secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
        Err(SecretStoreError::Unavailable {
            kind: SecretStoreKind::OsKeychain,
            message: "OS keychain backend is not available in this build".to_string(),
        })
    }

    fn health(&self) -> SecretStoreHealth {
        SecretStoreHealth::unavailable(
            SecretStoreKind::OsKeychain,
            vec![SecretStoreDiagnostic::recoverable(
                "secret_store.keychain_unavailable",
                "OS keychain backend is not available in this build; env fallback is available",
            )],
        )
    }
}

impl SecretResolver for OsKeychainSecretStore {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError> {
        self.get(secret_ref)
            .map_err(secret_store_to_resolution_error)
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

fn secret_ref_error(error: ModelConfigError) -> SecretStoreError {
    SecretStoreError::InvalidReference(error.to_string())
}

fn secret_store_to_resolution_error(error: SecretStoreError) -> SecretResolutionError {
    match error {
        SecretStoreError::SecretNotFound { provider, name } => {
            SecretResolutionError::SecretNotFound { provider, name }
        }
        SecretStoreError::EmptySecretValue { provider, name } => {
            SecretResolutionError::EmptySecretValue { provider, name }
        }
        SecretStoreError::InvalidReference(message)
        | SecretStoreError::Unavailable { message, .. } => {
            SecretResolutionError::UnsupportedSecretProvider { provider: message }
        }
        SecretStoreError::ReadOnly { kind } => SecretResolutionError::UnsupportedSecretProvider {
            provider: format!("{kind:?}"),
        },
    }
}
