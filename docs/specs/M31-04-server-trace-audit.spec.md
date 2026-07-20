# M31-04 Server Trace Audit Spec

## Purpose

Add trace-safe summaries for MCP server session frames so server behavior is inspectable without leaking tool input, resource contents, or secrets.

## Product Context

A long-running local server needs auditability. Users need to know what methods were served, which exports were touched, and whether policy allowed or denied access.

## Scope

This spec delivers:

- server frame trace entry contracts
- redacted session summary
- policy outcome recording for call/read paths
- CLI-readable trace snapshot helper

Owned by other specs:

- session execution: M31-01
- CLI stdio process: M31-03
- final docs: M31-05

## Architecture

`ruse-mcp::server_trace` owns trace structs and redaction. Session runtime emits trace entries; CLI renders summaries or returns them through status/debug surfaces later.

## File Layout

```text
crates/ruse-mcp/src/server_trace.rs
crates/ruse-mcp/tests/mcp_server_trace.rs
```

## Public Contracts

```rust
pub struct McpServerTraceEntry {
    pub method: String,
    pub outcome: String,
    pub initialized: bool,
}
```

## State and Failure Handling

- every handled frame can produce one trace entry
- malformed frames are represented without raw frame text
- denied policy decisions are visible as denied outcomes
- trace summaries remain bounded

## Security and Privacy

- no raw tool arguments
- no resource contents
- no raw secret-like identifiers
- request ids are bounded and redacted

## Testing Strategy

- trace serialization tests
- redaction tests
- policy denied trace tests
- session summary tests

## Acceptance Criteria

- `cargo test -p ruse-mcp mcp_server_trace` passes.
- trace entries are safe for CLI/Desktop display.

## Completion Signal

M31-04 is complete when server trace/audit contracts are implemented and integrated with session tests.
