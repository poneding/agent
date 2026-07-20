# M15-01 Model Runtime Resolver Spec

## Purpose

Resolve an `EffectiveRuntimeConfig` into a selected executable model provider from a registry before a turn starts.

## Product Context

M14 can produce effective config snapshots. M15-01 makes provider selection executable by checking that the selected provider is enabled, registered, and satisfies the capabilities required by a prompt turn.

## Scope

This spec delivers:

- `ruse-core::model_runtime`
- `ModelRuntimeResolver`
- `ConfiguredModelRuntime`
- `ModelRuntimeError`
- selected provider/model resolution from `EffectiveRuntimeConfig`
- capability checks against `ModelProviderRegistry`

Owned by later specs:

- retry/fallback invocation execution
- configured turn orchestration
- CLI prompt rendering

## Architecture

The resolver lives in `ruse-core` because it composes runtime config with model registry contracts. It does not parse files, resolve secrets, or call model providers.

## Public Contracts

```rust
pub struct ConfiguredModelRuntime {
    pub provider_id: String,
    pub model: String,
    pub provider: Arc<dyn ModelProvider>,
    pub capabilities: ModelCapabilities,
    pub invocation_policy: ModelInvocationPolicy,
}

pub struct ModelRuntimeResolver;

impl ModelRuntimeResolver {
    pub fn resolve(
        effective: &EffectiveRuntimeConfig,
        registry: &ModelProviderRegistry,
        requirements: &ModelRequestRequirements,
    ) -> Result<ConfiguredModelRuntime, ModelRuntimeError>;
}
```

## State And Failure Handling

- missing selected provider returns `ModelRuntimeError::ProviderNotRegistered`
- disabled selected provider returns `ModelRuntimeError::ProviderDisabled`
- missing capabilities return `ModelRuntimeError::MissingCapabilities`
- empty provider/model values return `ModelRuntimeError::InvalidSelection`

## Security And Privacy

- resolver diagnostics include provider/model ids only
- resolver never exposes secret refs or secret values
- resolver never invokes transport or network paths

## Testing Strategy

- resolves selected fake provider from effective config
- rejects missing registry provider
- rejects disabled selected provider
- rejects provider missing required capabilities
- error serialization/debug output excludes secret-like values

## Acceptance Criteria

- Core can turn effective config into an executable provider handle.
- Failures are typed and safe for CLI/Desktop diagnostics.

## Completion Signal

M15-01 is complete when `cargo test -p ruse-core --test model_runtime_resolver` and workspace gates pass, status advances to M15-02, and the slice is committed.
