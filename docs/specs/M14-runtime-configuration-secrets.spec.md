# M14 Runtime Configuration And Secrets Spec

## Purpose

Add durable runtime configuration loading, secret resolution contracts, config layering, and UI-safe diagnostics so provider settings from M13 can be used by CLI/Desktop without storing or printing raw credentials.

## Product Context

M13 created provider configuration and adapter contracts. M14 makes those contracts operational as runtime settings: users can load config files, override model choices from CLI/Desktop, validate secret references, and inspect the effective configuration safely before later milestones wire live model invocation into full turns.

## Scope

This milestone delivers:

- secret resolver contracts with redacted secret values
- YAML runtime config schema for model catalog and invocation policy
- direct provider `apiKey` support in the default local config path
- config layering for built-in, user, workspace, session, and explicit overrides
- CLI config flags and JSON diagnostics
- UI bridge config snapshot that Desktop can consume without secrets

Owned by later specs:

- live HTTP model invocation
- OS keychain and encrypted local secret stores
- interactive secret prompts
- native Desktop settings screens
- full config migration beyond schema version 1

Implementation primarily touches:

- `ruse-model`
- `ruse-core`
- `ruse-cli`
- `ruse-ui-bridge`
- `ruse-desktop`
- `docs/milestones/README.md`

## Dependencies

- M13 provider config, registry, and invocation policy contracts
- `docs/architecture/configuration.md`
- `docs/architecture/security-policy.md`
- `docs/architecture/crate-boundaries.md`
- `docs/architecture/testing-strategy.md`

## Architecture

`ruse-model` owns secret reference resolution contracts because they validate model-provider authentication without knowing runtime or UI behavior. `ruse-core` owns runtime configuration loading and layering because it composes model config with runtime policy. CLI and Desktop expose only safe summaries through typed command/output and UI bridge contracts.

```mermaid
flowchart TD
  cli["ruse-cli"] --> core["ruse-core config"]
  desktop["ruse-desktop"] --> bridge["ruse-ui-bridge"]
  bridge --> core
  core --> model["ruse-model config/secrets"]
  core --> storage["ruse-storage migrations"]
```

## Key Contracts

- `SecretResolver`
- `SecretValue`
- `InMemorySecretResolver`
- `EnvSecretResolver`
- `AgentRuntimeConfig`
- `RuntimeModelConfig`
- `ConfigLayer`
- `ConfigOverride`
- `EffectiveRuntimeConfig`
- `ConfigDiagnostic`
- `UiConfigSnapshot`

Contract rules:

- raw secret values never implement normal `Debug`, `Display`, `Serialize`, or UI conversion
- the default config file is `~/.config/agent/config.yaml`; legacy `~/.config/agent/agent.toml` is a compatibility fallback with diagnostics
- config files may contain direct provider `apiKey` values for local setup or `SecretRef` handles for advanced setups
- direct `apiKey` values are converted into redacted in-memory secret resolver entries before runtime invocation
- config diagnostics use field paths and provider ids, not credential contents
- explicit CLI/Desktop overrides have the highest precedence
- unknown config keys are rejected with typed diagnostics

## State And Failure Handling

- missing config files fail before runtime execution unless a caller explicitly uses defaults
- invalid YAML or explicit legacy TOML fails with `AgentConfigError::Parse`
- invalid provider catalogs fail with model config errors wrapped in config diagnostics
- missing secret references are recoverable diagnostics until a caller requires strict validation
- explicit overrides can replace provider/model selection without mutating lower-priority layers
- every effective config has a schema version and source layer summary

## Security And Privacy

- tests must include secret-like fixture values and prove they are absent from diagnostics, JSON frames, and snapshots
- environment variable resolver reads by name only and returns redacted wrappers
- config snapshots expose credential presence only; they never expose direct `apiKey` values or local secret ref names
- config loading does not broaden permissions or enable tools/plugins by default

## Testing Strategy

Required coverage:

- secret value redaction and non-serialization tests
- in-memory and environment resolver tests
- YAML config parse, direct `apiKey`, validation, unknown-key, and error-path tests
- legacy TOML fallback and migration-diagnostic tests
- layering precedence tests
- CLI `--config` and `--config-check --json` tests
- UI snapshot serialization tests
- workspace gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Users can load a model-provider config file with direct `apiKey` values or advanced secret refs without leaking credentials.
- Runtime code can produce an effective config from layered sources and overrides.
- CLI can validate config and print UI-safe JSON diagnostics.
- Desktop-facing contracts can display model/config status without accessing secret values.
- M15 can consume the effective config for runtime model invocation without redesigning configuration.

## Completion Signal

M14 is complete when all slice plans are checked off, focused and workspace verification pass, M14 status is marked complete, and active focus advances to the next milestone.
