# M14-03 Config Layering Spec

## Purpose

Compose multiple runtime config sources into one effective config with deterministic precedence and explicit override diagnostics.

## Product Context

The architecture defines layered configuration. M14-03 implements the model-domain subset so CLI flags, session settings, workspace files, and defaults can be combined safely.

## Scope

This spec delivers:

- `ConfigLayer`
- `ConfigSource`
- `ConfigOverride`
- `EffectiveRuntimeConfig`
- deterministic model provider merging by id
- explicit provider/model override support

Owned by later specs:

- tool, MCP, scheduler, plugin, and memory config domains
- persistent user/workspace config discovery
- policy-aware config mutation commands

## Architecture

Layering lives in `agent-core::config`. It consumes already-parsed `AgentRuntimeConfig` values and returns an effective immutable snapshot.

## Public Contracts

```rust
pub enum ConfigLayerKind {
    BuiltIn,
    User,
    Workspace,
    Project,
    Session,
    Task,
    Explicit,
}

pub struct ConfigLayer {
    pub kind: ConfigLayerKind,
    pub source: String,
    pub config: AgentRuntimeConfig,
}

pub struct ConfigOverride {
    pub provider: Option<String>,
    pub model: Option<String>,
}

pub struct EffectiveRuntimeConfig {
    pub config: AgentRuntimeConfig,
    pub selected_provider: String,
    pub selected_model: String,
    pub layers: Vec<ConfigLayerSummary>,
    pub diagnostics: Vec<ConfigDiagnostic>,
}
```

`config.terminal.theme` is selected through the same layer precedence as the model domain, so higher-priority user/workspace/session layers can override lower-priority terminal defaults without CLI presentation code parsing YAML or legacy TOML directly.

## State And Failure Handling

- higher-priority provider entries replace lower-priority entries by id
- disabled providers remain visible but cannot be selected
- explicit provider override must target an enabled provider
- explicit model override must be non-empty
- conflicting layers generate diagnostics but deterministic output

## Security And Privacy

- layer summaries include source names and provider ids only
- secret references remain handles
- no raw secret values are accepted or produced

## Testing Strategy

- higher-priority layer overrides default provider
- provider entries merge by id deterministically
- higher-priority layer overrides terminal theme
- explicit provider/model overrides win
- selecting disabled provider fails
- layer summaries serialize without secret values

## Acceptance Criteria

- Runtime can build one effective model config from ordered layers.
- CLI/Desktop overrides have a stable contract.
- Diagnostics explain precedence decisions without leaking secrets.

## Completion Signal

M14-03 is complete when `cargo test -p agent-core --test runtime_config_layering` and the workspace gates pass, status advances to M14-04, and the slice is committed.
