# M14-04 CLI Config Diagnostics Spec

## Purpose

Expose runtime configuration loading and validation through CLI flags and JSON diagnostics without invoking a live model provider.

## Product Context

Users need to verify config files and secret references before running real provider-backed turns. CLI must provide the same safe facts later consumed by Desktop.

## Scope

This spec delivers:

- `--config <path>`
- `--config-check`
- JSON config diagnostics frames
- provider/model override integration with loaded config
- safe error messages for invalid config

Owned by later specs:

- config mutation commands
- interactive secret prompting
- live model execution from config

## Architecture

CLI parses flags and delegates validation to `agent-core::config`. CLI renders text or `UiJsonFrame` output only; it does not parse YAML/TOML internals or resolve secret values.

## Public Contracts

- `CliInvocation` carries optional config path and config-check mode.
- `CliError` maps config parse/load failures into stable messages.
- JSON mode emits `config_diagnostic` and `config_summary` frames.

## State And Failure Handling

- missing `--config` value returns `CliError::MissingValue`
- `--config-check` without `--config` validates built-in defaults
- invalid config exits with status 2 in binary tests
- JSON diagnostics remain parseable newline-delimited frames

## Security And Privacy

- CLI output never contains raw secret-looking values
- JSON diagnostics may include secret reference provider/name only
- config-check performs no network calls

## Testing Strategy

- CLI parses `--config` and `--config-check`
- text mode reports valid config summary
- JSON mode emits config summary and diagnostics frames
- invalid YAML or explicit legacy TOML exits with non-zero status
- secret-like fixture values are absent from stdout/stderr

## Acceptance Criteria

- Users can run CLI config validation before model invocation exists.
- Config-check output is stable for automation and Desktop bridge reuse.
- Existing CLI prompt behavior remains compatible.

## Completion Signal

M14-04 is complete when `cargo test -p agent-cli --test config_diagnostics` and the workspace gates pass, status advances to M14-05, and the slice is committed.
