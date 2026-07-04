# M11 Plugin Runtime Spec

## Purpose

Open the runtime to controlled third-party extension through manifests, process isolation, declared capabilities, tools, hooks, and a WASM prototype.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is plugin manifests, version compatibility, process plugin host, capability validation, plugin tools, lifecycle hooks, and constrained WASM execution.

## Scope

This milestone delivers:

- PluginManifest contract
- PluginCompatibility contract
- PluginProcessHost contract
- PluginCapability contract
- PluginHook contract

Implementation primarily touches:

- `agent-plugin`
- `agent-tool`
- `agent-policy`
- `agent-sandbox`
- `agent-core`
- `agent-types`

## Dependencies

- M02 Tool, Policy, and Audit Kernel
- M06 JSON Mode and UI Bridge

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- PluginManifest contract
- PluginCompatibility contract
- PluginProcessHost contract
- PluginCapability contract
- PluginHook contract

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

- manifest parser tests
- compatibility matrix tests
- process plugin fixture tests
- capability validation tests
- plugin tool policy tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- A plugin can register a tool through a declared capability.
- Plugin tool calls pass through the same policy, sandbox, event, and audit path as built-in tools.

## Completion Signal

M11 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
