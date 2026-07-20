# M26-03 CLI Observability Commands Spec

## Purpose

Expose observability evidence through script-safe CLI commands with text and JSON outputs.

## Product Context

CLI users need quick trace summaries during development and JSON evidence bundles for debugging, review, support, and automation.

## Scope

This spec delivers:

- CLI command parsing for observability evidence
- text summary rendering
- JSON bundle output
- local fixture-backed smoke tests
- secret leakage tests

Owned by other specs:

- evidence builder: `M26-02`
- UI snapshots: `M26-04`

## Architecture

`ruse-cli` invokes `ruse-core` evidence APIs and renders outputs. It must not duplicate evidence generation logic.

## Public Contracts

```text
ruse-cli --observability-summary
ruse-cli --observability-json
```

## State and Failure Handling

- commands are read-only
- JSON output is newline-terminated and machine-readable
- invalid runtime evidence options produce non-zero exit output
- local fixture failures are shown as CLI diagnostics

## Security and Privacy

- default text output is summary-only
- JSON output uses already-redacted evidence bundle contracts
- tests assert secret-like values do not appear in stdout or stderr

## Testing Strategy

- command parser tests
- CLI process smoke tests
- JSON shape tests
- no-secret-leakage tests

## Acceptance Criteria

- CLI can print evidence summaries and JSON evidence for the local coding fixture.
- `cargo test -p ruse-cli observability` passes.

## Completion Signal

M26-03 is complete when CLI tests pass, status docs are updated, and the slice is committed.
