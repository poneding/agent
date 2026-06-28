# M30-02 Tool Export Registry Spec

## Purpose

Expose selected local tools as MCP `tools/list` and `tools/call` entries while preserving local policy boundaries.

## Product Context

Agent tools are already permissioned locally. MCP server mode must not create a second unsafe execution path; exported tools must still use local tool definitions and permission checks.

## Scope

This spec delivers:

- tool export descriptors
- export registry with deterministic ordering
- conversion from `agent-tool::ToolDefinition` to MCP tool descriptors
- policy-aware `tools/call` fixture behavior

Owned by other specs:

- core runtime audit event wiring: later M30 or follow-up milestone
- stdio frame handling: M30-04

## Architecture

`agent-mcp::server_tool` maps local tool definitions into MCP server-visible contracts. Execution uses `agent-tool` and policy request contracts; no direct tool execution bypass is added.

## Public Contracts

```rust
pub struct McpToolExport {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub enabled: bool,
}
```

## State and Failure Handling

- duplicate export names are rejected
- disabled exports are omitted from list responses
- unknown tool calls return not-found errors
- denied calls return errors without executing the tool
- tool output is bounded before serialization

## Security and Privacy

- exported descriptions and schemas are untrusted display data
- inputs are redacted in diagnostics
- side-effecting calls require local policy allow/lease

## Testing Strategy

- export list ordering tests
- duplicate/disabled tests
- denied call no-execution tests
- allowed fixture call output bounding tests

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_tool_export` passes.
- exported local tools remain policy gated.

## Completion Signal

M30-02 is complete when tool export listing and fixture calls are deterministic, safe, and tested.
