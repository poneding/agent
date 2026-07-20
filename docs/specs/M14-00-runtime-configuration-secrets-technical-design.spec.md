# M14-00 Runtime Configuration And Secrets Technical Design Spec

## Purpose

Define the M14 configuration and secret-handling baseline so runtime settings can become durable and inspectable without leaking credentials or coupling UI clients to model-provider internals.

## Product Context

M14 follows M13. Provider configs, registries, and invocation policies exist, but they are not yet loaded from user/workspace files or exposed as safe runtime settings. This design creates the bridge between static provider contracts and later live invocation.

## Scope

This design governs:

- secret resolver boundaries
- redacted secret value handling
- runtime config schema version 1
- YAML default parse and validation behavior, with legacy TOML fallback support
- config source layering and override precedence
- CLI config diagnostics
- UI-safe config snapshots

Owned by implementation slices:

- `M14-01-secret-resolver`
- `M14-02-config-file-loader`
- `M14-03-config-layering`
- `M14-04-cli-config-diagnostics`
- `M14-05-ui-config-snapshot`

## Architecture

`ruse-model` adds a `secrets` module with provider-neutral secret reference resolution. It does not read config files and does not emit runtime events. `ruse-core` adds a `config` module that resolves the default `~/.ruse/config.yaml` path, parses YAML or explicit legacy TOML, validates model settings, layers sources, converts direct provider `apiKey` values into an in-memory resolver, and emits diagnostics. `ruse-cli`, `ruse-ui-bridge`, and `ruse-desktop` render or transport summaries only.

```mermaid
flowchart TD
  model["ruse-model::secrets"] --> model_config["ruse-model::config"]
  core_config["ruse-core::config"] --> model_config
  core_config --> secrets["SecretResolver trait"]
  cli["ruse-cli"] --> core_config
  bridge["ruse-ui-bridge"] --> core_config
  desktop["ruse-desktop"] --> bridge
```

Dependency rules:

- `ruse-model` must not depend on `ruse-core`, CLI, Desktop, storage, or UI bridge.
- `ruse-core::config` may depend on `ruse-model` and `ruse-storage` schema-version types.
- UI crates must not parse YAML/TOML or resolve secrets directly.
- config layer merging must be deterministic and testable in memory.

## Secret Model

`SecretValue` wraps raw secret text. It may expose the raw string only through an explicit method intended for future transports, such as `expose_secret()`. `Debug` and `Display` must render a redacted marker. `SecretValue` must not derive `Serialize`.

`SecretResolver` resolves a `SecretRef` into a `SecretValue`:

```rust
pub trait SecretResolver: Send + Sync {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretResolutionError>;
}
```

M14 provides:

- `InMemorySecretResolver` for tests and fixture wiring
- `EnvSecretResolver` for environment variable references
- strict missing-secret diagnostics

## Runtime Config Schema

M14 uses schema version 1 and YAML for the default local config at `~/.ruse/config.yaml`:

```yaml
schemaVersion: 1
model:
  defaultProvider: openai
  invocation:
    timeoutMs: 30000
    maxAttempts: 2
    retryableFailures: [timeout, transport, rate_limited]
    fallbackProviderIds: [fake]
  providers:
    - id: openai
      kind: openai-compatible
      defaultModel: gpt-4.1-mini
      enabled: true
      baseUrl: https://api.openai.com/v1
      apiKey: "sk-..."
```

Legacy `~/.ruse/agent.toml` and explicit `--config path/to/file.toml` remain readable for compatibility. When no explicit path is supplied, `config.yaml` wins; if it is absent and legacy `agent.toml` exists, the loader uses it and emits a migration diagnostic.

The loader maps this into M13 contracts:

- `ModelProviderCatalog`
- `ModelProviderConfig`
- `ModelEndpointConfig`
- `SecretRef` or direct `apiKey` converted into a synthetic in-memory secret reference
- `ModelInvocationPolicy`

## Layering Model

Layers merge in this order:

1. built-in defaults
2. user config
3. workspace config
4. project config
5. session overrides
6. task overrides
7. explicit CLI/Desktop overrides

M14 implements the model-domain subset. Later milestones can extend the same layer model to tools, memory, MCP, scheduler, and plugins.

Merge rules:

- absent fields inherit from lower-priority layers
- provider entries merge by `id`
- explicit disabled providers remain present but cannot be selected
- explicit provider/model overrides affect the current effective config only
- merging never serializes raw secret values

## Diagnostics

Diagnostics are structured:

```rust
pub struct ConfigDiagnostic {
    pub severity: ConfigDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub field_path: Option<String>,
    pub source: Option<String>,
}
```

Diagnostics can be serialized to CLI JSON and UI bridge snapshots. Messages may mention provider ids, but never direct `apiKey` values or secret reference names that would reveal local credential layout.

## Security And Privacy

- Raw secret values stay inside `SecretValue` after config loading.
- `SecretValue` debug/display output is redacted.
- YAML fixtures use synthetic provider ids and fake `apiKey` values.
- CLI JSON tests include values such as `sk-test-secret` and assert absence from output.
- Config validation does not perform network calls.

## Testing Strategy

Required verification:

- secret resolver unit tests
- config YAML parser tests plus legacy TOML fallback tests
- config layering tests
- CLI JSON diagnostics tests
- UI snapshot tests
- redaction tests for every serialized public output
- standard workspace gates

## Acceptance Criteria

- M14 has a stable config/secrets architecture baseline.
- Slice specs/plans reference this design.
- M14-01 can begin without unresolved ownership or privacy decisions.

## Completion Signal

M14 technical design is complete when this document and the slice plans define exact contracts, files, tests, and commit boundaries for the milestone.
