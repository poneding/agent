# M28-03 Golden Update Workflow Spec

## Purpose

Make golden updates explicit, typed, redacted, and reviewable so normal evaluation runs never rewrite expected outputs accidentally.

## Product Context

M27 comparison can return normalized expected output. M28 needs a workflow that writes or stages golden updates only when the user asks for update mode.

## Scope

This spec delivers:

- golden update request and result contracts
- suite-aware golden update behavior
- dry-run summary
- overwrite safeguards
- redacted normalized output

Owned by other specs:

- CLI flags and command rendering: M28-04
- `xtask` gate behavior: M28-05

## Architecture

`ruse-eval::golden` owns update result contracts. Suite commands call it after running scenarios.

## Public Contracts

```rust
pub struct GoldenUpdateReport {
    pub updated: usize,
    pub unchanged: usize,
    pub diagnostics: Vec<EvalDiagnostic>,
}
```

## State and Failure Handling

- update mode must be explicit
- dry-run never writes
- malformed existing goldens fail before overwrite
- updated files are deterministic and newline-terminated

## Security and Privacy

- update output uses normalized redacted reports
- paths cannot traverse outside suite root

## Testing Strategy

- dry-run update test
- write update test
- malformed existing golden test
- redaction test

## Acceptance Criteria

- suite golden updates are explicit and reviewable.

## Completion Signal

M28-03 is complete when update reports and file writes are covered by tests.
