# M26-05 Replay Evidence Verification Spec

## Purpose

Prove observability evidence can be regenerated from stored events and replay fixtures without mutating runtime state.

## Product Context

Auditability depends on support bundles being reproducible. Users and maintainers need confidence that trace/audit/evidence exports match stored event history.

## Scope

This spec delivers:

- replay evidence verification report
- fixture compatibility checks for evidence generation
- idempotency assertions for evidence export
- final M26 workspace gate

Owned by other specs:

- evidence bundle shape: `M26-02`
- CLI output commands: `M26-03`
- UI snapshots: `M26-04`

## Architecture

`agent-core` performs evidence verification using `agent-storage` replay fixtures and event stores. CLI may call the verifier after this slice, but verification logic stays in core.

## Public Contracts

```rust
pub struct EvidenceReplayVerificationReport {
    pub replayable: bool,
    pub source_event_count: usize,
    pub regenerated_trace_count: usize,
    pub regenerated_audit_count: usize,
    pub diagnostics: Vec<String>,
}
```

## State and Failure Handling

- verification is read-only
- fixture version mismatches return diagnostics
- regenerated evidence mismatches fail with typed report data
- repeated verification for the same fixture returns the same counts

## Security and Privacy

- verification reports contain counts and diagnostics only
- raw fixture payloads are not copied into report diagnostics
- secret-like fixture values remain redacted through export paths

## Testing Strategy

- M04 fixture evidence regeneration test
- local coding fixture idempotent evidence test
- mismatch diagnostics test
- full M26 smoke test

## Acceptance Criteria

- replay verification proves evidence can be regenerated from existing fixtures.
- `cargo xtask check` passes before M26 is marked complete.

## Completion Signal

M26-05 and M26 are complete when replay verification tests and workspace gates pass, milestone docs are updated, and the completion commit is created.
