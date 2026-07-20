# M15-04 CLI Prompt Runtime Spec

## Purpose

Make `agent-cli --prompt` execute a configured runtime turn instead of returning a static echo.

## Product Context

The CLI is the first usable surface. After M15-03, CLI can delegate prompt execution to core and render a real final response from the selected provider.

## Scope

This spec delivers:

- text-mode prompt execution through `ConfiguredTurnRunner`
- config loading through existing `--config`
- provider/model overrides through existing flags
- default fake-provider registry for local deterministic execution
- safe errors for unsupported selected providers

Owned by later specs:

- live HTTP transport
- interactive TTY streaming
- approval UI during tool loops

## Architecture

CLI parses flags and builds an effective config using `agent-core::config`. It registers fake providers for executable fake configs, calls `ConfiguredTurnRunner`, and renders the final message. CLI does not parse YAML/TOML internals or call provider transports directly.

## State And Failure Handling

- invalid config exits with status 2
- selected non-registered provider exits with status 2 and a safe diagnostic
- successful prompt exits with status 0 and final response text
- existing `--config-check` behavior remains unchanged

## Security And Privacy

- text output must not include raw secret-like fixture values
- unsupported provider errors include provider id only
- CLI must not read environment secret values for prompt execution in M15

## Testing Strategy

- `--prompt` runs default fake configured turn
- `--prompt --config fake.toml` uses configured fake model output
- `--prompt --config openai.toml` fails safely when no provider is registered
- config-check tests continue to pass

## Acceptance Criteria

- CLI prompt mode executes through runtime config.
- Existing CLI JSON/config diagnostics remain compatible.

## Completion Signal

M15-04 is complete when `cargo test -p agent-cli --test prompt_runtime` and workspace gates pass, status advances to M15-05, and the slice is committed.
