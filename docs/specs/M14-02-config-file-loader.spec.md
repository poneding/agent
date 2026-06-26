# M14-02 Config File Loader Spec

## Purpose

Add runtime config schema version 1 and a TOML loader that maps model configuration into M13 provider catalog and invocation policy contracts.

## Product Context

Users need durable, human-editable config files before runtime model choices can be reused across CLI/Desktop sessions.

## Scope

This spec delivers:

- `agent-core::config`
- `AgentRuntimeConfig`
- `RuntimeModelConfig`
- TOML parsing from string and file path
- schema version validation
- unknown-key rejection
- config diagnostics

Owned by later specs:

- multi-domain tool/plugin/memory config
- config migration beyond schema version 1
- live provider invocation

## Architecture

Config loading lives in `agent-core` because it composes runtime domains. The model subset maps into `agent-model` contracts and does not resolve secret values.

## Public Contracts

```rust
pub const AGENT_CONFIG_SCHEMA_VERSION: u32 = 1;

pub struct AgentRuntimeConfig {
    pub schema_version: u32,
    pub model: RuntimeModelConfig,
}

pub struct RuntimeModelConfig {
    pub default_provider: String,
    pub catalog: ModelProviderCatalog,
    pub invocation_policy: ModelInvocationPolicy,
}

pub fn load_agent_config_toml(input: &str) -> Result<AgentRuntimeConfig, AgentConfigError>;
pub fn load_agent_config_file(path: &Path) -> Result<AgentRuntimeConfig, AgentConfigError>;
```

## State And Failure Handling

- missing files return a typed file error
- invalid TOML returns a parse error
- unsupported schema versions return a version error
- invalid model catalog or policy returns validation diagnostics
- loader performs no network calls and no secret value resolution

## Security And Privacy

- TOML supports secret provider/name handles only
- raw secret-looking keys such as `api_key_value`, `token`, and `secret` are rejected
- diagnostics redact raw secret-looking values

## Testing Strategy

- valid TOML parses into model catalog and invocation policy
- unsupported schema version fails
- unknown keys fail
- raw secret value key fails
- invalid provider config produces field-path diagnostics
- file loader reports missing files

## Acceptance Criteria

- `agent-core` exports config loader contracts.
- Config fixtures round-trip into existing model contracts.
- Secret references remain handles after parsing.

## Completion Signal

M14-02 is complete when `cargo test -p agent-core --test runtime_config_loader` and the workspace gates pass, status advances to M14-03, and the slice is committed.
