# M17-03 Live Provider Registration Spec

## Purpose

Register executable live model providers from effective runtime config without moving provider-specific code into `agent-core`.

## Product Context

M14 and M15 load and resolve runtime config, but CLI currently registers only fake providers. After M17-02, OpenAI-compatible configs need to construct a real transport-backed provider when a prompt is executed.

## Scope

This slice delivers:

- CLI-side registry builder for enabled config providers
- fake provider registration preserved for deterministic local runs
- OpenAI-compatible provider registration with env secret resolver and reqwest transport
- missing secret diagnostics before model invocation
- test seams for fixture transports without live network

Out of scope:

- Desktop settings UI
- keychain-backed secrets
- provider marketplace discovery

## Architecture

`agent-cli` owns config-to-registry wiring for the CLI process. `agent-model` owns provider implementations. `agent-core` continues to accept a `ModelProviderRegistry` and never imports reqwest or provider-specific setup logic.

Dependency rules:

- `agent-cli` may depend on `agent-model` provider constructors.
- `agent-core` must not depend on reqwest.
- `agent-model` must not read CLI flags.

## State And Failure Handling

- Disabled providers are skipped.
- Fake providers remain available for tests and default config.
- OpenAI-compatible providers require a secret ref and resolver.
- Missing or empty env secret values fail before HTTP calls.
- Registration errors are rendered without raw secret values.

## Security And Privacy

Only secret references are stored in config. Raw secret values are resolved inside transport execution and are never copied into CLI output, model selection frames, or runtime events.

## Testing Strategy

- config containing fake and OpenAI-compatible providers registers both
- disabled providers are not registered
- missing env secret fails safely
- fake prompt runtime remains deterministic
- core tests prove no provider-specific dependency leaks into `agent-core`

## Acceptance Criteria

- CLI can build a live-capable provider registry from config.
- Missing secrets fail before network attempts.
- Provider-specific wiring stays outside core.

## Completion Signal

M17-03 is complete when CLI registry tests and workspace gates pass.
