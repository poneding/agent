# M14-02 Config File Loader Spec

## Purpose

Add runtime config schema version 1 and a YAML-first loader that maps model configuration into M13 provider catalog and invocation policy contracts.

## Product Context

Users need durable, human-editable config files before runtime model choices can be reused across CLI/Desktop sessions.

## Scope

This spec delivers:

- `agent-core::config`
- `AgentRuntimeConfig`
- `RuntimeModelConfig`
- `RuntimeTerminalConfig`
- YAML parsing from string and file path
- legacy TOML parsing for explicit files and `~/.config/agent/agent.toml` fallback
- default path resolution for `~/.config/agent/config.yaml`
- schema version validation
- unknown-key rejection
- config diagnostics

Owned by later specs:

- multi-domain tool/plugin/memory config
- terminal config beyond startup theme defaults
- config migration beyond schema version 1
- live provider invocation

## Architecture

Config loading lives in `agent-core` because it composes runtime domains. The model subset maps into `agent-model` contracts. Direct provider `apiKey` values are converted into an in-memory secret resolver during load; environment and other secret refs stay as handles.

## Public Contracts

```rust
pub const AGENT_CONFIG_SCHEMA_VERSION: u32 = 1;

pub struct AgentRuntimeConfig {
    pub schema_version: u32,
    pub model: RuntimeModelConfig,
    pub terminal: RuntimeTerminalConfig,
}

pub struct RuntimeModelConfig {
    pub default_provider: String,
    pub catalog: ModelProviderCatalog,
    pub invocation_policy: ModelInvocationPolicy,
}

pub struct RuntimeTerminalConfig {
    pub theme: RuntimeTerminalTheme,
}

pub enum RuntimeTerminalTheme {
    System,
    Light,
    Dark,
    HighContrast,
}

pub fn default_agent_config_path() -> Option<PathBuf>;
pub fn resolve_agent_config_path(
    explicit_path: Option<PathBuf>,
) -> Result<AgentConfigPathResolution, AgentConfigError>;
pub fn load_agent_config_yaml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError>;
pub fn load_agent_config_yaml_with_secrets(input: &str) -> Result<LoadedAgentConfig, AgentConfigError>;
pub fn load_agent_config_toml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError>;
pub fn load_agent_config_file(path: &Path) -> Result<AgentRuntimeConfig, AgentConfigError>;
```

## State And Failure Handling

- missing files return a typed file error
- invalid YAML or explicit legacy TOML returns a parse error
- absent `config.yaml` falls back to legacy `agent.toml` only when no explicit path is supplied, and emits a migration diagnostic
- unsupported schema versions return a version error
- invalid model catalog or policy returns validation diagnostics
- loader performs no network calls and no secret value resolution

## Security And Privacy

- YAML supports direct provider `apiKey` and secret provider/name handles
- direct `apiKey` values are stored only in a redacted in-memory resolver for runtime use
- raw secret-looking keys such as `api_key_value`, `token`, and `secret` are rejected
- diagnostics redact raw secret-looking values

## Testing Strategy

- valid YAML parses into model catalog and invocation policy
- valid YAML parses direct provider `apiKey` into the inline secret resolver
- valid YAML parses optional terminal startup theme
- legacy TOML parses when explicitly supplied or selected by fallback
- unsupported schema version fails
- unknown keys fail
- raw secret value key fails
- invalid provider config produces field-path diagnostics
- file loader reports missing files
- default-path resolution prefers `config.yaml` and reports legacy fallback diagnostics

## Acceptance Criteria

- `agent-core` exports config loader contracts.
- Config fixtures map into existing model contracts.
- Direct `apiKey` values and secret references remain redacted after parsing.

## Completion Signal

M14-02 is complete when `cargo test -p agent-core --test runtime_config_loader` and the workspace gates pass, status advances to M14-03, and the slice is committed.
