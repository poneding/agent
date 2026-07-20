use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ruse_model::{
    InMemorySecretResolver, ModelConfigError, ModelEndpointConfig, ModelFailureKind,
    ModelFallbackChain, ModelInvocationPolicy, ModelInvocationPolicyError, ModelProviderCatalog,
    ModelProviderConfig, ModelProviderKind, ModelRetryPolicy, SecretRef,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const AGENT_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const AGENT_CONFIG_FILE_NAME: &str = "config.yaml";
pub const AGENT_LEGACY_CONFIG_FILE_NAME: &str = "agent.toml";
const INLINE_CONFIG_SECRET_PROVIDER: &str = "config";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedAgentConfig {
    pub config: AgentRuntimeConfig,
    pub inline_secret_resolver: InMemorySecretResolver,
    pub diagnostics: Vec<ConfigDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentConfigPathResolution {
    pub load_path: Option<PathBuf>,
    pub write_path: PathBuf,
    pub diagnostics: Vec<ConfigDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub schema_version: u32,
    pub model: RuntimeModelConfig,
    pub terminal: RuntimeTerminalConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeModelConfig {
    pub default_provider: String,
    pub catalog: ModelProviderCatalog,
    pub invocation_policy: ModelInvocationPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeTerminalConfig {
    pub theme: RuntimeTerminalTheme,
}

impl Default for RuntimeTerminalConfig {
    fn default() -> Self {
        Self {
            theme: RuntimeTerminalTheme::System,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeTerminalTheme {
    System,
    Light,
    Dark,
    HighContrast,
}

impl RuntimeTerminalTheme {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::HighContrast => "high-contrast",
        }
    }
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
    pub fn error(
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

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        field_path: Option<String>,
        source: Option<String>,
    ) -> Self {
        Self {
            severity: ConfigDiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            field_path,
            source,
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
    #[error("config file could not be written: {path}: {message}")]
    FileWrite { path: String, message: String },
    #[error("config file could not be parsed: {message}")]
    Parse { message: String },
    #[error("config file could not be serialized: {message}")]
    Serialize { message: String },
    #[error("unsupported config schema version {found}; supported version is {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("config is invalid: {diagnostics:?}")]
    InvalidConfig { diagnostics: Vec<ConfigDiagnostic> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigLayerKind {
    BuiltIn,
    User,
    Workspace,
    Project,
    Session,
    Task,
    Explicit,
}

impl ConfigLayerKind {
    const fn precedence(self) -> u8 {
        match self {
            Self::BuiltIn => 0,
            Self::User => 1,
            Self::Workspace => 2,
            Self::Project => 3,
            Self::Session => 4,
            Self::Task => 5,
            Self::Explicit => 6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigLayer {
    pub kind: ConfigLayerKind,
    pub source: String,
    pub config: AgentRuntimeConfig,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigOverride {
    pub provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConfigLayerSummary {
    pub kind: ConfigLayerKind,
    pub source: String,
    pub default_provider: String,
    pub provider_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EffectiveRuntimeConfig {
    pub config: AgentRuntimeConfig,
    pub selected_provider: String,
    pub selected_model: String,
    pub layers: Vec<ConfigLayerSummary>,
    pub diagnostics: Vec<ConfigDiagnostic>,
}

pub fn load_agent_config_toml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError> {
    Ok(load_agent_config_toml_with_secrets(input)?.config)
}

pub fn load_agent_config_toml_with_secrets(
    input: &str,
) -> Result<LoadedAgentConfig, AgentConfigError> {
    reject_raw_secret_value_keys(input)?;
    let raw: RawAgentConfig = toml::from_str(input).map_err(|error| AgentConfigError::Parse {
        message: error.to_string(),
    })?;
    raw.try_into_runtime()
}

pub fn load_agent_config_yaml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError> {
    Ok(load_agent_config_yaml_with_secrets(input)?.config)
}

pub fn load_agent_config_yaml_with_secrets(
    input: &str,
) -> Result<LoadedAgentConfig, AgentConfigError> {
    reject_raw_secret_value_keys_yaml(input)?;
    let raw: RawAgentConfig =
        serde_yaml::from_str(input).map_err(|error| AgentConfigError::Parse {
            message: error.to_string(),
        })?;
    raw.try_into_runtime()
}

pub fn load_agent_config_file(path: &Path) -> Result<AgentRuntimeConfig, AgentConfigError> {
    Ok(load_agent_config_file_with_secrets(path)?.config)
}

pub fn load_agent_config_file_with_secrets(
    path: &Path,
) -> Result<LoadedAgentConfig, AgentConfigError> {
    let input = std::fs::read_to_string(path).map_err(|error| AgentConfigError::FileRead {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    match config_file_format(path) {
        AgentConfigFileFormat::Toml => load_agent_config_toml_with_secrets(&input),
        AgentConfigFileFormat::Yaml => load_agent_config_yaml_with_secrets(&input),
        AgentConfigFileFormat::Unknown => load_agent_config_yaml_with_secrets(&input)
            .or_else(|_| load_agent_config_toml_with_secrets(&input)),
    }
}

#[must_use]
pub fn default_agent_config_path_for_home(home: &Path) -> PathBuf {
    home.join(".ruse").join(AGENT_CONFIG_FILE_NAME)
}

#[must_use]
pub fn legacy_agent_config_path_for_home(home: &Path) -> PathBuf {
    home.join(".ruse").join(AGENT_LEGACY_CONFIG_FILE_NAME)
}

#[must_use]
pub fn default_agent_config_path() -> Option<PathBuf> {
    current_home_dir().map(|home| default_agent_config_path_for_home(&home))
}

#[must_use]
pub fn legacy_agent_config_path() -> Option<PathBuf> {
    current_home_dir().map(|home| legacy_agent_config_path_for_home(&home))
}

pub fn resolve_agent_config_path(
    explicit_path: Option<PathBuf>,
) -> Result<AgentConfigPathResolution, AgentConfigError> {
    if let Some(path) = explicit_path {
        return Ok(AgentConfigPathResolution {
            load_path: Some(path.clone()),
            write_path: path,
            diagnostics: Vec::new(),
        });
    }

    let Some(default_path) = default_agent_config_path() else {
        return Err(AgentConfigError::FileRead {
            path: "~/.ruse/config.yaml".to_string(),
            message: "home directory could not be resolved".to_string(),
        });
    };
    Ok(resolve_agent_config_path_for_home_paths(
        default_path,
        legacy_agent_config_path(),
    ))
}

#[must_use]
pub fn resolve_agent_config_path_for_home(home: &Path) -> AgentConfigPathResolution {
    resolve_agent_config_path_for_home_paths(
        default_agent_config_path_for_home(home),
        Some(legacy_agent_config_path_for_home(home)),
    )
}

fn resolve_agent_config_path_for_home_paths(
    default_path: PathBuf,
    legacy_path: Option<PathBuf>,
) -> AgentConfigPathResolution {
    if default_path.is_file() {
        return AgentConfigPathResolution {
            load_path: Some(default_path.clone()),
            write_path: default_path,
            diagnostics: Vec::new(),
        };
    }

    let Some(legacy_path) = legacy_path else {
        return AgentConfigPathResolution {
            load_path: None,
            write_path: default_path.clone(),
            diagnostics: Vec::new(),
        };
    };
    if legacy_path.is_file() {
        return AgentConfigPathResolution {
            load_path: Some(legacy_path.clone()),
            write_path: default_path.clone(),
            diagnostics: vec![ConfigDiagnostic::warning(
                "config.legacy_toml_fallback",
                format!(
                    "loaded legacy {}; migrate runtime settings to {}",
                    legacy_path.display(),
                    default_path.display()
                ),
                None,
                Some(legacy_path.display().to_string()),
            )],
        };
    }

    AgentConfigPathResolution {
        load_path: None,
        write_path: default_path,
        diagnostics: Vec::new(),
    }
}

pub fn write_agent_config_terminal_theme_atomic(
    path: &Path,
    theme: RuntimeTerminalTheme,
) -> Result<(), AgentConfigError> {
    let input = std::fs::read_to_string(path).map_err(|error| AgentConfigError::FileRead {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let (serialized, format) = match config_file_format(path) {
        AgentConfigFileFormat::Toml => (
            write_terminal_theme_toml(&input, theme)?,
            AgentConfigFileFormat::Toml,
        ),
        AgentConfigFileFormat::Yaml => (
            write_terminal_theme_yaml(&input, theme)?,
            AgentConfigFileFormat::Yaml,
        ),
        AgentConfigFileFormat::Unknown => match write_terminal_theme_yaml(&input, theme) {
            Ok(serialized) => (serialized, AgentConfigFileFormat::Yaml),
            Err(_) => (
                write_terminal_theme_toml(&input, theme)?,
                AgentConfigFileFormat::Toml,
            ),
        },
    };
    load_agent_config_file_format(&serialized, format)?;
    write_config_atomic(path, serialized)
}

fn write_terminal_theme_toml(
    input: &str,
    theme: RuntimeTerminalTheme,
) -> Result<String, AgentConfigError> {
    let mut value: toml::Value =
        toml::from_str(input).map_err(|error| AgentConfigError::Parse {
            message: error.to_string(),
        })?;
    let Some(root) = value.as_table_mut() else {
        return Err(AgentConfigError::Parse {
            message: "config root must be a TOML table".to_string(),
        });
    };
    let terminal = root
        .entry("terminal".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let Some(terminal) = terminal.as_table_mut() else {
        return Err(invalid_config(ConfigDiagnostic::error(
            "terminal.invalid",
            "terminal config must be a table",
            Some("terminal".to_string()),
        )));
    };
    terminal.insert(
        "theme".to_string(),
        toml::Value::String(theme.as_str().to_string()),
    );
    toml::to_string_pretty(&value).map_err(|error| AgentConfigError::Serialize {
        message: error.to_string(),
    })
}

fn write_terminal_theme_yaml(
    input: &str,
    theme: RuntimeTerminalTheme,
) -> Result<String, AgentConfigError> {
    let mut value: serde_yaml::Value =
        serde_yaml::from_str(input).map_err(|error| AgentConfigError::Parse {
            message: error.to_string(),
        })?;
    let Some(root) = value.as_mapping_mut() else {
        return Err(AgentConfigError::Parse {
            message: "config root must be a YAML mapping".to_string(),
        });
    };
    let terminal_key = serde_yaml::Value::String("terminal".to_string());
    let terminal = root
        .entry(terminal_key)
        .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    let Some(terminal) = terminal.as_mapping_mut() else {
        return Err(invalid_config(ConfigDiagnostic::error(
            "terminal.invalid",
            "terminal config must be a mapping",
            Some("terminal".to_string()),
        )));
    };
    terminal.insert(
        serde_yaml::Value::String("theme".to_string()),
        serde_yaml::Value::String(theme.as_str().to_string()),
    );
    serde_yaml::to_string(&value).map_err(|error| AgentConfigError::Serialize {
        message: error.to_string(),
    })
}

fn load_agent_config_file_format(
    input: &str,
    format: AgentConfigFileFormat,
) -> Result<AgentRuntimeConfig, AgentConfigError> {
    match format {
        AgentConfigFileFormat::Toml => load_agent_config_toml(input),
        AgentConfigFileFormat::Yaml | AgentConfigFileFormat::Unknown => {
            load_agent_config_yaml(input)
        }
    }
}

fn write_config_atomic(path: &Path, serialized: String) -> Result<(), AgentConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| AgentConfigError::FileWrite {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serialized).map_err(|error| AgentConfigError::FileWrite {
        path: tmp.display().to_string(),
        message: error.to_string(),
    })?;
    std::fs::rename(&tmp, path).map_err(|error| AgentConfigError::FileWrite {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(())
}

pub fn build_effective_runtime_config(
    layers: Vec<ConfigLayer>,
    config_override: ConfigOverride,
) -> Result<EffectiveRuntimeConfig, AgentConfigError> {
    if layers.is_empty() {
        return Err(invalid_config(ConfigDiagnostic::error(
            "config.layers.empty",
            "at least one config layer is required",
            Some("layers".to_string()),
        )));
    }

    let mut ordered_layers = layers.into_iter().enumerate().collect::<Vec<_>>();
    ordered_layers.sort_by_key(|(index, layer)| (layer.kind.precedence(), *index));

    let mut schema_version = AGENT_CONFIG_SCHEMA_VERSION;
    let mut default_provider = String::new();
    let mut invocation_policy = None;
    let mut selected_terminal = RuntimeTerminalConfig::default();
    let mut providers = BTreeMap::new();
    let mut summaries = Vec::with_capacity(ordered_layers.len());
    let mut diagnostics = Vec::new();

    for (_, layer) in ordered_layers {
        schema_version = layer.config.schema_version;
        default_provider = layer.config.model.default_provider.clone();
        invocation_policy = Some(layer.config.model.invocation_policy.clone());
        selected_terminal = layer.config.terminal.clone();
        summaries.push(layer_summary(&layer));

        for provider in layer.config.model.catalog.providers {
            let provider_id = provider.id.clone();
            if providers.insert(provider_id.clone(), provider).is_some() {
                diagnostics.push(ConfigDiagnostic::warning(
                    "config.layer.provider_overridden",
                    format!("provider {provider_id} was overridden by a higher-priority layer"),
                    Some(format!("model.providers.{provider_id}")),
                    Some(layer.source.clone()),
                ));
            }
        }
    }

    let catalog =
        ModelProviderCatalog::new(providers.into_values().collect()).map_err(|error| {
            invalid_config(model_catalog_diagnostic(
                error,
                "model.providers".to_string(),
            ))
        })?;
    let invocation_policy = invocation_policy.expect("non-empty layers set invocation policy");
    let config = AgentRuntimeConfig {
        schema_version,
        model: RuntimeModelConfig {
            default_provider,
            catalog,
            invocation_policy,
        },
        terminal: selected_terminal,
    };
    let selected_provider = config_override
        .provider
        .unwrap_or_else(|| config.model.default_provider.clone())
        .trim()
        .to_string();
    if selected_provider.is_empty() {
        return Err(invalid_config(selection_diagnostic(
            "selected provider must not be empty",
            "model.selected_provider",
        )));
    }
    let provider = config
        .model
        .catalog
        .providers
        .iter()
        .find(|provider| provider.id == selected_provider)
        .ok_or_else(|| {
            invalid_config(selection_diagnostic(
                format!("selected provider {selected_provider} was not found"),
                "model.selected_provider",
            ))
        })?;
    if !provider.enabled {
        return Err(invalid_config(selection_diagnostic(
            format!("selected provider {selected_provider} is disabled"),
            "model.selected_provider",
        )));
    }
    let selected_model = if let Some(model) = config_override.model {
        let model = model.trim().to_string();
        if model.is_empty() {
            return Err(invalid_config(selection_diagnostic(
                "selected model must not be empty",
                "model.selected_model",
            )));
        }
        model
    } else {
        provider.default_model.clone()
    };

    Ok(EffectiveRuntimeConfig {
        config,
        selected_provider,
        selected_model,
        layers: summaries,
        diagnostics,
    })
}

fn layer_summary(layer: &ConfigLayer) -> ConfigLayerSummary {
    let mut provider_ids = layer
        .config
        .model
        .catalog
        .providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<Vec<_>>();
    provider_ids.sort();

    ConfigLayerSummary {
        kind: layer.kind,
        source: layer.source.clone(),
        default_provider: layer.config.model.default_provider.clone(),
        provider_ids,
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAgentConfig {
    #[serde(alias = "schemaVersion")]
    schema_version: u32,
    model: RawModelConfig,
    #[serde(default)]
    terminal: RawTerminalConfig,
}

impl RawAgentConfig {
    fn try_into_runtime(self) -> Result<LoadedAgentConfig, AgentConfigError> {
        if self.schema_version != AGENT_CONFIG_SCHEMA_VERSION {
            return Err(AgentConfigError::UnsupportedSchemaVersion {
                found: self.schema_version,
                supported: AGENT_CONFIG_SCHEMA_VERSION,
            });
        }

        let model = self.model.try_into_runtime()?;
        let config = AgentRuntimeConfig {
            schema_version: self.schema_version,
            model: model.config,
            terminal: self.terminal.try_into_runtime(),
        };
        Ok(LoadedAgentConfig {
            config,
            inline_secret_resolver: model.inline_secret_resolver,
            diagnostics: model.diagnostics,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTerminalConfig {
    #[serde(default)]
    theme: Option<RuntimeTerminalTheme>,
}

impl RawTerminalConfig {
    fn try_into_runtime(self) -> RuntimeTerminalConfig {
        RuntimeTerminalConfig {
            theme: self.theme.unwrap_or(RuntimeTerminalTheme::System),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModelConfig {
    #[serde(alias = "defaultProvider")]
    default_provider: String,
    invocation: RawInvocationConfig,
    providers: Vec<RawProviderConfig>,
}

struct RuntimeModelBuild {
    config: RuntimeModelConfig,
    inline_secret_resolver: InMemorySecretResolver,
    diagnostics: Vec<ConfigDiagnostic>,
}

impl RawModelConfig {
    fn try_into_runtime(self) -> Result<RuntimeModelBuild, AgentConfigError> {
        if self.default_provider.trim().is_empty() {
            return Err(invalid_config(ConfigDiagnostic::error(
                "model.default_provider.invalid",
                "model.default_provider must not be empty",
                Some("model.default_provider".to_string()),
            )));
        }

        let invocation_policy = self.invocation.try_into_policy()?;
        let mut providers = Vec::with_capacity(self.providers.len());
        let mut inline_secret_resolver = InMemorySecretResolver::new();
        let mut diagnostics = Vec::new();
        for (index, provider) in self.providers.into_iter().enumerate() {
            let provider = provider.try_into_provider(
                index,
                invocation_policy.timeout_ms,
                &mut inline_secret_resolver,
                &mut diagnostics,
            )?;
            providers.push(provider);
        }
        let catalog = ModelProviderCatalog::new(providers).map_err(|error| {
            invalid_config(model_catalog_diagnostic(
                error,
                "model.providers".to_string(),
            ))
        })?;

        Ok(RuntimeModelBuild {
            config: RuntimeModelConfig {
                default_provider: self.default_provider,
                catalog,
                invocation_policy,
            },
            inline_secret_resolver,
            diagnostics,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInvocationConfig {
    #[serde(alias = "timeoutMs")]
    timeout_ms: u64,
    #[serde(alias = "maxAttempts")]
    max_attempts: u32,
    #[serde(alias = "retryableFailures")]
    retryable_failures: Vec<ModelFailureKind>,
    #[serde(alias = "fallbackProviderIds")]
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
    #[serde(alias = "defaultModel")]
    default_model: String,
    enabled: bool,
    #[serde(default, alias = "baseUrl")]
    base_url: Option<String>,
    #[serde(default, alias = "timeoutMs")]
    timeout_ms: Option<u64>,
    #[serde(default, alias = "apiKey")]
    api_key: Option<String>,
    #[serde(default, alias = "apiKeyProvider")]
    api_key_provider: Option<String>,
    #[serde(default, alias = "apiKeyName")]
    api_key_name: Option<String>,
}

impl RawProviderConfig {
    fn try_into_provider(
        self,
        index: usize,
        default_timeout_ms: u64,
        inline_secret_resolver: &mut InMemorySecretResolver,
        diagnostics: &mut Vec<ConfigDiagnostic>,
    ) -> Result<ModelProviderConfig, AgentConfigError> {
        let field = |name: &str| format!("model.providers[{index}].{name}");
        let api_key = match (self.api_key, self.api_key_provider, self.api_key_name) {
            (Some(value), provider, name) => {
                if provider.is_some() || name.is_some() {
                    diagnostics.push(ConfigDiagnostic::warning(
                        "model.provider.api_key.precedence",
                        "provider apiKey is configured directly, so apiKeyProvider/apiKeyName were ignored",
                        Some(field("apiKey")),
                        None,
                    ));
                }
                let secret_ref = inline_api_key_secret_ref(index, &self.id)?;
                inline_secret_resolver
                    .insert_secret(secret_ref.clone(), value)
                    .map_err(|error| {
                        invalid_config(ConfigDiagnostic::error(
                            "model.provider.api_key.invalid",
                            format!("provider apiKey is invalid: {error}"),
                            Some(field("apiKey")),
                        ))
                    })?;
                Some(secret_ref)
            }
            (None, Some(provider), Some(name)) => {
                Some(SecretRef::new(provider, name).map_err(|error| {
                    invalid_config(model_config_diagnostic(error, field("apiKey")))
                })?)
            }
            (None, None, None) => None,
            _ => {
                return Err(invalid_config(ConfigDiagnostic::error(
                    "model.provider.api_key.invalid",
                    "apiKeyProvider and apiKeyName must be set together",
                    Some(field("apiKey")),
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

fn reject_raw_secret_value_keys_yaml(input: &str) -> Result<(), AgentConfigError> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(input).map_err(|error| AgentConfigError::Parse {
            message: error.to_string(),
        })?;
    let mut diagnostics = Vec::new();
    collect_raw_secret_keys_yaml(&value, "", &mut diagnostics);
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

fn collect_raw_secret_keys_yaml(
    value: &serde_yaml::Value,
    path: &str,
    diagnostics: &mut Vec<ConfigDiagnostic>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, nested) in map {
                let Some(key) = key.as_str() else {
                    continue;
                };
                let nested_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                if is_raw_secret_value_key(key) {
                    diagnostics.push(ConfigDiagnostic::error(
                        "config.raw_secret_key",
                        "raw secret value keys are not allowed; use apiKey or secret references",
                        Some(nested_path.clone()),
                    ));
                }
                collect_raw_secret_keys_yaml(nested, &nested_path, diagnostics);
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for (index, nested) in items.iter().enumerate() {
                collect_raw_secret_keys_yaml(nested, &format!("{path}[{index}]"), diagnostics);
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

fn selection_diagnostic(message: impl Into<String>, field_path: &str) -> ConfigDiagnostic {
    ConfigDiagnostic::error(
        "config.selection.invalid",
        message,
        Some(field_path.to_string()),
    )
}

fn inline_api_key_secret_ref(
    index: usize,
    provider_id: &str,
) -> Result<SecretRef, AgentConfigError> {
    let provider = provider_id.trim();
    let name = if provider.is_empty() {
        format!("provider-{index}")
    } else {
        format!("provider-{provider}")
    };
    SecretRef::new(INLINE_CONFIG_SECRET_PROVIDER, name).map_err(|error| {
        invalid_config(ConfigDiagnostic::error(
            "model.provider.api_key.invalid",
            format!("provider apiKey could not be registered: {error}"),
            Some(format!("model.providers[{index}].apiKey")),
        ))
    })
}

fn field_for_model_error(index: usize, error: &ModelConfigError) -> String {
    let field = match error {
        ModelConfigError::EmptyProviderId | ModelConfigError::InvalidProviderId { .. } => "id",
        ModelConfigError::EmptyDefaultModel { .. } => "default_model",
        ModelConfigError::InvalidTimeout { .. } => "timeout_ms",
        ModelConfigError::MissingEndpoint { .. } | ModelConfigError::UnsafeEndpointUrl { .. } => {
            "base_url"
        }
        ModelConfigError::MissingSecretRef { .. }
        | ModelConfigError::EmptySecretProvider
        | ModelConfigError::InvalidSecretProvider { .. }
        | ModelConfigError::EmptySecretName => "apiKey",
        ModelConfigError::DuplicateProviderId { .. } | ModelConfigError::NoEnabledProviders => {
            "providers"
        }
    };
    format!("model.providers[{index}].{field}")
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AgentConfigFileFormat {
    Toml,
    Yaml,
    Unknown,
}

fn config_file_format(path: &Path) -> AgentConfigFileFormat {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("toml") => AgentConfigFileFormat::Toml,
        Some("yaml" | "yml") => AgentConfigFileFormat::Yaml,
        _ => AgentConfigFileFormat::Unknown,
    }
}

fn current_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
