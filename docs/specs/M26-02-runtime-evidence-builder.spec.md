# M26-02 Runtime Evidence Builder Spec

## Purpose

Compose trace entries, audit export bundles, telemetry export bundles, and replay metadata into a single `ObservabilityEvidenceBundle`.

## Product Context

Users should not need to manually join trace output, audit JSON, telemetry records, and replay diagnostics. Core runtime should produce one typed evidence object that every surface can consume.

## Scope

This spec delivers:

- `ObservabilityEvidenceBundle`
- `ObservabilityEvidenceOptions`
- `ReplayEvidenceSummary`
- runtime evidence builder in `ruse-core`
- local coding fixture evidence test

Owned by other specs:

- telemetry export primitives: `M26-01`
- CLI rendering and JSON commands: `M26-03`
- UI bridge snapshots: `M26-04`
- replay verification command coverage: `M26-05`

## Architecture

`ruse-core` owns evidence composition because it already owns trace queries and audit export. Storage remains responsible for event retrieval; evidence builder accepts typed events and telemetry records.

## Public Contracts

```rust
pub fn build_observability_evidence(
    events: &[RuntimeEvent],
    telemetry: &[TelemetryRecord],
    options: ObservabilityEvidenceOptions,
) -> Result<ObservabilityEvidenceBundle, ObservabilityEvidenceError>;
```

## State and Failure Handling

- empty event and telemetry inputs produce an empty bundle with diagnostics
- invalid payload limits fail before any bundle is built
- missing checkpoint or replay data appears as replay diagnostics
- audit and telemetry export errors are wrapped with typed core errors

## Security and Privacy

- builder uses audit and telemetry redaction paths instead of duplicating raw serialization
- trace summaries remain bounded and redacted
- generated evidence includes schema versions for later migration

## Testing Strategy

- evidence bundle serde shape
- local coding fixture evidence summary
- invalid options
- no raw secret leakage across trace, audit, and telemetry sections

## Acceptance Criteria

- `ruse-core` can build one evidence bundle from runtime events and telemetry records.
- `cargo test -p ruse-core observability_evidence` passes.

## Completion Signal

M26-02 is complete when evidence builder tests pass, status docs are updated, and the slice is committed.
