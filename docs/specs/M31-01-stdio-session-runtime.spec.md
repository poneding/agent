# M31-01 Stdio Session Runtime Spec

## Purpose

Add an in-memory MCP server session runtime that processes line-framed JSON-RPC requests repeatedly and reports deterministic lifecycle state.

## Product Context

The M30 dispatcher handles one frame. A real stdio server needs a session wrapper that preserves initialization state, enforces frame limits, and returns structured shutdown summaries.

## Scope

This spec delivers:

- `McpServerSession` contract
- session config and status
- repeated frame handling over in-memory inputs
- deterministic close summary

Owned by other specs:

- export profile construction: M31-02
- CLI process IO: M31-03
- trace summaries: M31-04

## Architecture

`ruse-mcp::server_session` wraps `McpServerFixture` and `handle_mcp_server_frame`. It does not read stdin or write stdout directly.

## File Layout

```text
crates/ruse-mcp/src/server_session.rs
crates/ruse-mcp/tests/mcp_server_session.rs
```

## Public Contracts

```rust
pub struct McpServerSessionConfig {
    pub max_frame_bytes: usize,
    pub max_frames: Option<u64>,
}
```

## State and Failure Handling

- `Created` transitions to `Initialized` when initialize succeeds
- malformed frames increment error counts without closing the session
- max-frame violations return bounded errors
- frame limit closes the session gracefully

## Security and Privacy

- oversize input is never echoed
- errors reuse M30 redaction behavior
- session summary contains counts and method names only

## Testing Strategy

- state transition tests
- malformed frame recovery tests
- frame limit tests
- oversized frame tests

## Acceptance Criteria

- `cargo test -p ruse-mcp mcp_server_session` passes.
- session runtime has no direct process IO dependency.

## Completion Signal

M31-01 is complete when session contracts, lifecycle behavior, and focused tests are implemented.
