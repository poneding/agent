use std::path::Path;

use agent_model::{
    ModelConfigError, ModelEndpointConfig, ModelFailureKind, ModelFallbackChain,
    ModelInvocationPolicy, ModelInvocationPolicyError, ModelProviderCatalog, ModelProviderConfig,
    ModelProviderKind, ModelRetryPolicy, SecretRef,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const AGENT_CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub schema_version: u32,
    pub model: RuntimeModelConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeModelConfig {
    pub default_provider: String,
    pub catalog: ModelProviderCatalog,
    pub invocation_policy: ModelInvocationPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigDiagnostic {
    pub severity: ConfigDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field_path: Option<String>,
    pub source: Option<String>,
}

impl ConfigDiagnostic {
    fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
    ) -> Self {
        Self {
            severity: ConfigDiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            field_path,
            source: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AgentConfigError {
    #[error("config file could not be read: {path}: {message}")]
    FileRead { path: String, message: String },
    #[error("config TOML could not be parsed: {message}")]
    Parse { message: String },
    #[error("unsupported config schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("config is invalid: {diagnostics:?}")]
    InvalidConfig { diagnostics: Vec<ConfigDiagnostic> },
}

pub fn load_agent_config_toml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError> {
    reject_raw_secret_value_keys(input)?;
    let raw: RawAgentConfig = toml::from_str(input).map_err(|error| AgentConfigError::Parse {
        message: error.to_string(),
    })?;
    raw.try_into_runtime()
}

pub fn load_agent_config_file(path: &Path) -> Result<AgentRuntimeConfig, AgentConfigError> {
    let input = std::fs::read_to_string(path).map_err(|error| AgentConfigError::FileRead {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    load_agent_config_toml(&input)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAgentConfig {
    schema_version: u32,
    model: RawModelConfig,
}

impl RawAgentConfig {
    fn try_into_runtime(self) -> Result<AgentRuntimeConfig, AgentConfigError> {
        if self.schema_version != AGENT_CONFIG_SCHEMA_VERSION {
            return Err(AgentConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: AGENT_CONFIG_SCHEMA_VERSION,
            });
        }

        Ok(AgentRuntimeConfig {
            schema_version: self.schema_version,
            model: self.model.try_into_runtime()?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModelConfig {
    default_provider: String,
    invocation: RawInvocationConfig,
    providers: Vec<RawProviderConfig>,
}

impl RawModelConfig {
    fn try_into_runtime(self) -> Result<RuntimeModelConfig, AgentConfigError> {
        if self.default_provider.trim().is_empty() {
            return Err(invalid_config(ConfigDiagnostic::error(
                "model.default_provider.invalid",
                "model.default_provider must not be empty",
                Some("model.default_provider".to_string()),
            )));
        }

        let invocation_policy = self.invocation.try_into_policy()?;
        let mut providers = Vec::with_capacity(self.providers.len());
        for (index, provider) in self.providers.into_iter().enumerate() {
            providers.push(provider.try_into_provider(index, invocation_policy.timeout_ms)?);
        }
        let catalog = ModelProviderCatalog::new(providers).map_err(|error| {
            invalid_config(model_catalog_diagnostic(
                error,
                "model.providers".to_string(),
            ))
        })?;

        Ok(RuntimeModelConfig {
            default_provider: self.default_provider,
            catalog,
            invocation_policy,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInvocationConfig {
    timeout_ms: u64,
    max_attempts: u32,
    retryable_failures: Vec<ModelFailureKind>,
    fallback_provider_ids: Vec<String>,
}

impl RawInvocationConfig {
    fn try_into_policy(self) -> Result<ModelInvocationPolicy, AgentConfigError> {
        let retry =
            ModelRetryPolicy::new(self.max_attempts, self.retryable_failures).map_err(|error| {
                invalid_config(invocation_diagnostic(
                    error,
                    "model.invocation.max_attempts".to_string(),
                ))
            })?;
        let fallback = ModelFallbackChain::new(self.fallback_provider_ids).map_err(|error| {
            invalid_config(invocation_diagnostic(
                error,
                "model.invocation.fallback_provider_ids".to_string(),
            ))
        })?;
        ModelInvocationPolicy::new(self.timeout_ms, retry, fallback).map_err(|error| {
            invalid_config(invocation_diagnostic(
                error,
                "model.invocation.timeout_ms".to_string(),
            ))
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProviderConfig {
    id: String,
    kind: ModelProviderKind,
    default_model: String,
    enabled: bool,
    base_url: Option<String>,
    timeout_ms: Option<u64>,
    api_key_provider: Option<String>,
    api_key_name: Option<String>,
}

impl RawProviderConfig {
    fn try_into_provider(
        self,
        index: usize,
        default_timeout_ms: u64,
    ) -> Result<ModelProviderConfig, AgentConfigError> {
        let field = |name: &str| format!("model.providers[{index}].{name}");
        let api_key = match (self.api_key_provider, self.api_key_name) {
            (Some(provider), Some(name)) => {
                Some(SecretRef::new(provider, name).map_err(|error| {
                    invalid_config(model_config_diagnostic(error, field("api_key")))
                })?)
            }
            (None, None) => None,
            _ => {
                return Err(invalid_config(ConfigDiagnostic::error(
                    "model.provider.api_key.invalid",
                    "api_key_provider and api_key_name must be set together",
                    Some(field("api_key")),
                )));
            }
        };
        let endpoint = ModelEndpointConfig::new(
            self.base_url.as_deref(),
            self.timeout_ms.unwrap_or(default_timeout_ms),
        )
        .map_err(|error| invalid_config(model_config_diagnostic(error, field("base_url"))))?;
        ModelProviderConfig::new(
            self.id,
            self.kind,
            self.default_model,
            endpoint,
            api_key,
            self.enabled,
        )
        .map_err(|error| {
            let field_path = field_for_model_error(index, &error);
            invalid_config(model_config_diagnostic(error, field_path))
        })
    }
}

fn reject_raw_secret_value_keys(input: &str) -> Result<(), AgentConfigError> {
    let value: toml::Value = toml::from_str(input).map_err(|error| AgentConfigError::Parse {
        message: error.to_string(),
    })?;
    let mut diagnostics = Vec::new();
    collect_raw_secret_keys(&value, "", &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(AgentConfigError::InvalidConfig { diagnostics })
    }
}

fn collect_raw_secret_keys(
    value: &toml::Value,
    path: &str,
    diagnostics: &mut Vec<ConfigDiagnostic>,
) {
    match value {
        toml::Value::Table(table) => {
            for (key, nested) in table {
                let nested_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                if is_raw_secret_value_key(key) {
                    diagnostics.push(ConfigDiagnostic::error(
                        "config.raw_secret_key",
                        "raw secret value keys are not allowed; use secret references",
                        Some(nested_path.clone()),
                    ));
                }
                collect_raw_secret_keys(nested, &nested_path, diagnostics);
            }
        }
        toml::Value::Array(items) => {
            for (index, nested) in items.iter().enumerate() {
                collect_raw_secret_keys(nested, &format!("{path}[{index}]"), diagnostics);
            }
        }
        _ => {}
    }
}

fn is_raw_secret_value_key(key: &str) -> bool {
    matches!(key, "api_key_value" | "token" | "secret")
}

fn invalid_config(diagnostic: ConfigDiagnostic) -> AgentConfigError {
    AgentConfigError::InvalidConfig {
        diagnostics: vec![diagnostic],
    }
}

fn model_config_diagnostic(error: ModelConfigError, field_path: String) -> ConfigDiagnostic {
    ConfigDiagnostic::error(
        "model.provider.invalid",
        format!("invalid model provider config: {error}"),
        Some(field_path),
    )
}

fn model_catalog_diagnostic(error: ModelConfigError, field_path: String) -> ConfigDiagnostic {
    ConfigDiagnostic::error(
        "model.catalog.invalid",
        format!("invalid model provider catalog: {error}"),
        Some(field_path),
    )
}

fn invocation_diagnostic(
    error: ModelInvocationPolicyError,
    field_path: String,
) -> ConfigDiagnostic {
    ConfigDiagnostic::error(
        "model.invocation.invalid",
        format!("invalid model invocation policy: {error}"),
        Some(field_path),
    )
}

fn field_for_model_error(index: usize, error: &ModelConfigError) -> String {
    let field = match error {
        ModelConfigError::EmptyProviderId | ModelConfigError::InvalidProviderId { .. } => "id",
        ModelConfigError::EmptyDefaultModel { .. } => "default_model",
        ModelConfigError::InvalidTimeout { .. } => "timeout_ms",
        ModelConfigError::UnsafeEndpointUrl { .. } => "base_url",
        ModelConfigError::MissingSecretRef { .. }
        | ModelConfigError::EmptySecretProvider
        | ModelConfigError::InvalidSecretProvider { .. }
        | ModelConfigError::EmptySecretName => "api_key",
        ModelConfigError::DuplicateProviderId { .. } | ModelConfigError::NoEnabledProviders => {
            "providers"
        }
    };
    format!("model.providers[{index}].{field}")
}
