# M31-02 Export Profile Bootstrap Spec

## Purpose

Add an explicit MCP server export profile that builds safe default tool/resource export registries for stdio server sessions.

## Product Context

Server mode must not expose capabilities implicitly. Users and tests need one profile object that describes what can be exported before the server starts.

## Scope

This spec delivers:

- export profile contracts
- default-deny empty profile
- fixture profile for deterministic tests
- validation and redaction rules

Owned by other specs:

- session runtime: M31-01
- CLI flags/config loading: M31-03
- trace summaries: M31-04

## Architecture

`agent-mcp::server_profile` maps profile records into `McpToolExportRegistry`, `McpResourceExportRegistry`, `ToolRegistry`, and `PolicyEngine` fixtures. Later config-backed profiles can reuse this boundary.

## File Layout

```text
crates/agent-mcp/src/server_profile.rs
crates/agent-mcp/tests/mcp_server_profile.rs
```

## Public Contracts

```rust
pub struct McpServerExportProfile {
    pub name: String,
    pub tool_names: Vec<String>,
    pub resource_uris: Vec<String>,
}
```

## State and Failure Handling

- empty profile is valid and exports nothing
- duplicate entries are rejected
- disabled entries are omitted from runtime registries
- invalid or secret-like identifiers fail validation

## Security and Privacy

- profile diagnostics never include resource contents
- secret-like names/URIs are redacted
- all exported executable tools still require policy decisions

## Testing Strategy

- default profile tests
- duplicate/invalid validation tests
- deterministic fixture profile tests
- redaction tests

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_profile` passes.
- stdio sessions can be constructed from a profile without implicit exports.

## Completion Signal

M31-02 is complete when export profile contracts and fixture bootstrap tests are implemented.
