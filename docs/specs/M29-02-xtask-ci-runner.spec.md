# M29-02 Xtask CI Runner Spec

## Purpose

Add `cargo xtask ci` as the one local and CI entrypoint for quality gates.

## Product Context

Developers should not have to remember separate check and eval commands. CI should execute the same gate and receive a typed report.

## Scope

This spec delivers:

- `Task::Ci`
- deterministic quality gate command plan
- command execution with report updates
- non-zero exit propagation

Owned by other specs:

- report schema: M29-01
- hosted workflow: M29-03

## Architecture

`xtask ci` runs Cargo commands and `cargo xtask eval`. It writes reports through M29-01 contracts.

## Public Contracts

```rust
pub fn run_quality_gate(options: QualityGateOptions) -> Result<QualityGateReport, QualityGateError>;
```

## State and Failure Handling

- fmt, clippy, test, and eval run in that order
- first failure stops remaining default steps
- report is written before returning failure

## Security and Privacy

- no command logs in JSON
- no environment variables serialized
- artifact paths are deterministic

## Testing Strategy

- parser test for `ci`
- command plan test
- fixture command failure test
- artifact writer test

## Acceptance Criteria

- `cargo xtask ci` runs locally and writes `target/agent-ci` artifacts.

## Completion Signal

M29-02 is complete when the shared CI runner is implemented and tested.
