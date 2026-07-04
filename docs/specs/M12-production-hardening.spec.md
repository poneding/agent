# M12 Production Hardening Spec

## Purpose

Make the runtime reliable for daily use and external integration with migrations, telemetry, audit export, memory portability, platform reports, and benchmarks.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is schema migrations, config migrations, OpenTelemetry, audit export, memory import/export, platform capability reports, and repeatable benchmarks.

## Scope

This milestone delivers:

- SchemaMigration contract
- TelemetrySpan contract
- AuditExport contract
- MemoryArchive contract
- PlatformCapabilityReport contract
- BenchmarkScenario contract

Implementation primarily touches:

- `agent-storage`
- `agent-telemetry`
- `agent-memory`
- `agent-sandbox`
- `agent-core`
- `agent-cli`

## Dependencies

- M01 through M11 milestone deliverables

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- SchemaMigration contract
- TelemetrySpan contract
- AuditExport contract
- MemoryArchive contract
- PlatformCapabilityReport contract
- BenchmarkScenario contract

Contract rules:

- Public data crossing crate boundaries uses explicit Rust types.
- Durable data includes a schema or compatibility strategy.
- Side-effecting behavior emits runtime events or audit records.
- UI-facing behavior is expressed through events or bridge contracts rather than direct UI calls.

## State and Failure Handling

- State transitions are explicit and test-covered.
- Recoverable failures become typed runtime errors with stable codes.
- Non-recoverable failures stop the current task or turn with persisted diagnostic context.
- Retried work must be idempotent or guarded by a recorded lease, checkpoint, or correlation identifier.

## Security and Privacy

- Capabilities are explicit at crate and runtime boundaries.
- Secrets are never persisted in plain runtime events, snapshots, trace bundles, or test fixtures.
- External input from models, repositories, plugins, MCP servers, tools, and UI clients is treated as untrusted.
- Audit-relevant decisions include actor, scope, resource, action, decision, and reason.

## Testing Strategy

Required test coverage:

- migration fixture tests
- telemetry field tests
- audit export tests
- memory archive tests
- platform report tests
- benchmark smoke tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Users can upgrade data safely.
- Runtime behavior is observable in logs, metrics, traces, and audit exports.
- Core benchmarks are tracked and repeatable.

## Completion Signal

M12 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
