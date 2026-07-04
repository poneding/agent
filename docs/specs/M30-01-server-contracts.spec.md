# M30-01 Server Contracts Spec

## Purpose

Add typed MCP server protocol contracts for initialization, capabilities, JSON-RPC requests, responses, and errors.

## Product Context

Tool and resource export need stable protocol shapes before executable behavior is added. This slice creates deterministic server contracts without spawning a real server process.

## Scope

This spec delivers:

- `agent-mcp::server` module
- server info and capability structs
- JSON-RPC request, response, and error contracts
- method enum for supported server operations

Owned by other specs:

- tool export execution: M30-02
- resource export execution: M30-03
- stdio frame loop: M30-04

## Architecture

Contracts live in `agent-mcp` and are serde-friendly. They do not depend on CLI, Desktop, storage, or runtime orchestration.

## File Layout

```text
crates/agent-mcp/
  src/server.rs
  src/lib.rs
  tests/mcp_server_contracts.rs
```

## Public Contracts

```rust
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub protocol_version: String,
}

pub enum McpServerMethod {
    Initialize,
    ToolsList,
    ToolsCall,
    ResourcesList,
    ResourcesRead,
}
```

## State and Failure Handling

- invalid server names, versions, and protocol versions are rejected
- unsupported methods return a typed method error
- JSON-RPC ids preserve string, number, or null shape
- error messages are bounded and redacted

## Security and Privacy

- protocol metadata must not include secrets
- parse errors do not echo raw oversized payloads
- capabilities advertise only implemented exports

## Testing Strategy

- stable serde shape tests
- validation tests for empty/secret-like metadata
- method parsing tests
- JSON-RPC response/error rendering tests

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_contracts` passes.
- public contracts are exported from `agent_mcp`.

## Completion Signal

M30-01 is complete when protocol contracts are implemented, tested, and milestone status advances to M30-02.
