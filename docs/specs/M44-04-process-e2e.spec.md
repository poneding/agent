# M44-04 Process E2E Spec

## Purpose

Verify live subprocess MCP stdio tool execution from initialize through gateway-registered local tool execution.

## Product Context

Fixture transport proves protocol mapping, but M44 must also prove the spawned-process transport works for `tools/call` in the same runtime path users will rely on.

## Scope

This spec delivers:

- local helper process serving initialize, tools/list, and tools/call
- `McpStdioClient::spawn` E2E execution test
- gateway discover/register/execute over spawned process
- process close and cleanup verification through existing close path

Owned by other specs:

- final docs and workspace gate

## Architecture

Tests live under `crates/agent-mcp/tests/` and compile a small Rust helper process at test time, matching M43 process test style.

## Public Contracts

- `McpStdioClient::spawn`
- `McpGateway::discover_tools`
- `McpGateway::register_tools`

## State and Failure Handling

- Helper processes receive ordered JSON-RPC ids over one session.
- EOF and close are handled by M43 process transport.

## Security and Privacy

- Test helper stderr remains non-protocol and must not be required for success.
- Public assertions avoid raw secret output in diagnostics.

## Testing Strategy

- subprocess E2E test named under `mcp_client_stdio_tool_call_process`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call_process`

## Completion Signal

M44-04 is complete when the spawned-process E2E can call a remote MCP tool through the gateway.
