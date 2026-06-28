# M30-04 Stdio JSON-RPC Fixture Spec

## Purpose

Add deterministic line-framed JSON-RPC request handling for MCP server tests and future stdio server mode.

## Product Context

Local MCP server mode usually communicates over stdio. This slice proves request parsing and response rendering without introducing a long-running production process.

## Scope

This spec delivers:

- request dispatcher for initialized server fixtures
- line-framed JSON-RPC parse/render helpers
- initialize, tools/list, tools/call, resources/list, resources/read dispatch
- malformed frame handling

Owned by other specs:

- protocol contracts: M30-01
- tool/resource registries: M30-02 and M30-03
- CLI command: M30-05

## Architecture

Frame handling lives in `agent-mcp`. It accepts strings/bytes and returns response strings for tests. Production process management remains outside this slice.

## Public Contracts

```rust
pub fn handle_mcp_server_frame(
    server: &mut McpServerFixture,
    frame: &str,
) -> String;
```

## State and Failure Handling

- parse errors return JSON-RPC parse errors
- unknown methods return method-not-found errors
- invalid params return invalid-params errors
- dispatcher state remains valid after malformed frames

## Security and Privacy

- frames are size-bounded
- errors do not echo raw oversized input
- exported tool/resource behavior remains policy-gated

## Testing Strategy

- initialize frame tests
- list and call frame tests
- malformed JSON tests
- state-after-error tests

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_stdio` passes.
- frame handling is deterministic and does not require network or subprocesses.

## Completion Signal

M30-04 is complete when stdio-style fixture handling covers supported server methods and error paths.
