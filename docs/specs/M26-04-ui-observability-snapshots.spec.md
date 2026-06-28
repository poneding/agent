# M26-04 UI Observability Snapshots Spec

## Purpose

Normalize observability evidence into bounded UI bridge snapshots for TUI and Desktop consumers.

## Product Context

Desktop and TUI need trace/audit/telemetry/replay status without parsing full support bundles or exposing raw data.

## Scope

This spec delivers:

- UI-safe observability snapshot contracts
- trace/audit/telemetry/replay summary cards
- bounded diagnostics
- Desktop-facing smoke tests

Owned by other specs:

- core evidence bundle generation: `M26-02`
- CLI command rendering: `M26-03`

## Architecture

`agent-ui-bridge` owns UI-safe frame contracts. `agent-desktop` and `agent-tui` consume snapshots and render them without mutating runtime evidence.

## Public Contracts

```rust
pub struct UiObservabilitySnapshot {
    pub schema_version: u32,
    pub trace_count: usize,
    pub audit_count: usize,
    pub telemetry_count: usize,
    pub replayable: bool,
    pub diagnostics: Vec<String>,
}
```

## State and Failure Handling

- oversized diagnostics are truncated
- empty evidence produces an empty snapshot
- unsupported evidence versions produce safe diagnostics

## Security and Privacy

- snapshots contain counts, statuses, and bounded summaries only
- raw telemetry attributes are omitted from default UI snapshots
- UI bridge tests assert no raw secret values appear in serialized snapshots

## Testing Strategy

- UI bridge snapshot serde tests
- Desktop/TUI smoke rendering tests
- diagnostics bounding tests
- no-secret-leakage tests

## Acceptance Criteria

- UI bridge can produce a Desktop/TUI-safe observability snapshot from evidence.
- `cargo test -p agent-ui-bridge observability` passes.

## Completion Signal

M26-04 is complete when UI snapshot tests pass, status docs are updated, and the slice is committed.
