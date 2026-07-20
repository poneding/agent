# M50-01 Stream Supervisor Contracts Spec

## Purpose

Add pure contracts for MCP HTTP stream supervision configuration, lifecycle status, and safe validation.

## Product Context

Before a scheduler or UI can manage stream supervision, the runtime needs typed, serializable contracts that define what can be supervised and how status is reported.

## Scope

This spec delivers:

- supervisor config contract
- lifecycle status enum
- validation rules for bounded supervision options
- stable serde shape tests

Owned by other specs:

- runner behavior in M50-02
- cancellation/lease behavior in M50-03
- rich reports in M50-04

## Architecture

Contracts live in `ruse-mcp::client_http` and are exported through `ruse-mcp`.

## Public Contracts

```rust
pub struct McpClientHttpStreamSupervisorConfig {
    pub server_id: McpServerId,
    pub reconnect_policy: McpClientHttpReconnectPolicy,
    pub max_ticks: u32,
}
```

## State and Failure Handling

- `max_ticks` must be greater than zero.
- reconnect policy validation reuses M49 behavior.
- validation errors are public and redacted.

## Security and Privacy

- Config validation does not expose URLs, headers, session ids, event ids, or secrets.
- Contracts grant no new local permissions.

## Testing Strategy

- valid default config
- invalid bounds
- invalid reconnect policy propagation
- stable serde shape

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p ruse-mcp mcp_client_http_stream_supervisor_contracts`
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m50 cargo clippy -p ruse-mcp --all-targets -- -D warnings`

## Completion Signal

M50-01 is complete when supervisor contracts are typed, exported, validated, redacted, and covered by focused tests.
