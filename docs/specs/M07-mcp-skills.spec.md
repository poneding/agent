# M07 MCP and Skills Spec

## Purpose

Add standard external capability discovery through MCP servers and skill roots.

## Product Context

This milestone advances the agent toward the PRD goal of a modular, auditable, recoverable, provider-agnostic runtime shared by CLI and Desktop. Its design center is MCP client configuration, tool discovery, policy mapping, skill metadata, prompt contribution, and traceability.

## Scope

This milestone delivers:

- McpServerConfig contract
- McpToolDescriptor contract
- McpPolicyMapping contract
- SkillManifest contract
- SkillPromptContribution contract

Implementation is split across:

- `M07-01-mcp-client`
- `M07-02-mcp-policy-mapping`
- `M07-03-skill-discovery`
- `M07-04-mcp-skill-traceability`

Implementation primarily touches:

- `agent-mcp`
- `agent-skill`
- `agent-tool`
- `agent-policy`
- `agent-context`
- `agent-types`

## Dependencies

- M02 Tool, Policy, and Audit Kernel
- M05 Context and Memory MVP

## Architecture

The milestone keeps the runtime layered: shared contracts live in lower-level crates, orchestration remains in `agent-core`, and presentation behavior stays outside core runtime crates. Each public contract must be typed, serializable when it crosses process or storage boundaries, and testable without a live model provider.

## Key Contracts

- McpServerConfig contract
- McpToolDescriptor contract
- McpPolicyMapping contract
- SkillManifest contract
- SkillPromptContribution contract

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

- MCP discovery fixture tests
- MCP policy mapping tests
- skill manifest parser tests
- skill prompt source tracking tests

Required verification commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Acceptance Criteria

- Runtime discovers MCP tools and exposes them as permissioned local tools.
- Skills influence context assembly with traceable source metadata.

## Completion Signal

M07 is complete when all acceptance criteria pass, the implementation is covered by tests, and the milestone plan has no unchecked implementation tasks.
