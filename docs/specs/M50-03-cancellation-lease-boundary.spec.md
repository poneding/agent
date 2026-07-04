# M50-03 Cancellation Lease Boundary Spec

## Purpose

Add typed cancellation and lease preflight boundaries for MCP HTTP stream supervision.

## Product Context

Background supervision must cooperate with scheduler leases and user cancellation. This slice keeps cancellation explicit and testable without coupling MCP protocol logic directly to scheduler internals.

## Scope

This spec delivers:

- cancellation signal contract
- lease snapshot/preflight contract
- stop-before-side-effect behavior
- redacted cancellation/lease diagnostics

Owned by other specs:

- scheduler queue implementation outside `agent-mcp`
- Desktop controls and notifications outside this slice

## Architecture

`agent-mcp` consumes lightweight lease/cancel DTOs. `agent-scheduler` can map durable leases into these DTOs later without reverse dependencies.

## Public Contracts

```rust
pub struct McpClientHttpStreamSupervisorControl {
    pub cancelled: bool,
    pub lease_valid: bool,
}
```

Concrete names may change during implementation.

## State and Failure Handling

- Cancelled supervision returns `Cancelled` before opening a stream.
- Invalid lease returns a safe failure before opening a stream.
- No checkpoint mutation occurs when preflight stops execution.

## Security and Privacy

- Cancellation and lease diagnostics must not expose scheduler internals, user ids, task ids with sensitive content, headers, or event ids.

## Testing Strategy

- cancellation before side effects
- invalid lease before side effects
- valid lease proceeds to runner
- redacted diagnostics

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_cancellation`
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p agent-mcp --all-targets -- -D warnings`

## Completion Signal

M50-03 is complete when supervisor execution can be stopped safely before stream side effects by cancellation or lease preflight.
