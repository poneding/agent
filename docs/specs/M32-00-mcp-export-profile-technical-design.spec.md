# M32-00 MCP Export Profile Technical Design Spec

## Purpose

Define how explicit export manifests become runnable MCP stdio server profiles without moving runtime logic into CLI code.

## Product Context

Users need to expose selected local capabilities to MCP clients. The runtime must keep exports explicit, policy-gated, and auditable.

## Scope

This spec delivers:

- manifest schema and validation baseline
- profile construction boundaries
- resource and tool loading ownership
- CLI stdio flag ownership
- privacy and verification rules

Owned by other specs:

- concrete manifest contracts: M32-01
- resource file loading: M32-02
- built-in tool export catalog: M32-03
- CLI profile flags: M32-04
- final docs: M32-05

## Architecture

`ruse-mcp` parses and validates manifests, then constructs `McpServerProfileRuntime`. `ruse-cli` passes the manifest path/profile name and starts `McpServerSession`.

## File Layout

```text
crates/ruse-mcp/src/server_manifest.rs
crates/ruse-mcp/src/server_profile.rs
crates/ruse-cli/src/command.rs
```

## Public Contracts

The technical design commits to versioned serde contracts and a profile selector that fails before server startup.

## State and Failure Handling

- manifest validation is all-or-nothing
- profile selection is deterministic by name
- invalid exports return redacted `McpError`
- default empty profile remains available

## Security and Privacy

- reject secret-like identifiers
- do not print resource contents in diagnostics
- keep policy checks on call/read
- keep stdout protocol-only once stdio starts

## Testing Strategy

- serde snapshot-style tests
- duplicate and secret rejection tests
- CLI subprocess tests for invalid manifest stderr/stdout separation

## Acceptance Criteria

- M32 slice specs and plans exist.
- ownership and failure boundaries are explicit.

## Completion Signal

This design is complete when M32 has executable slice specs and plans.
