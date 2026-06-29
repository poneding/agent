# M44-03 Tool Call Redaction and Limits Spec

## Purpose

Harden MCP stdio tool-call failures and outputs so public diagnostics remain bounded and free of local secrets.

## Product Context

Remote MCP servers are untrusted. Tool-call support must not leak local arguments, secret refs, process stderr, or raw remote failure messages through errors.

## Scope

This spec delivers:

- tests for remote JSON-RPC failure redaction
- tests for malformed success result diagnostics
- tests that local secret-like input is redacted before gateway calls
- output bounding verification remains local gateway behavior

Owned by other specs:

- process E2E happy path
- final docs

## Architecture

Redaction stays in `agent-mcp::client_stdio` for client mapping errors and in `agent-mcp::gateway` for local argument handling.

## Public Contracts

- Public `McpError` messages for tool-call failures are bounded and generic.
- Remote success output remains returned data.

## State and Failure Handling

- JSON-RPC failures do not expose remote `error.message`.
- Invalid result shapes do not expose raw payload content.
- Transport errors reuse existing M43 process/session diagnostics.

## Security and Privacy

- No raw local tool argument appears in diagnostics.
- Secret-like keys and values are redacted before gateway call dispatch.

## Testing Strategy

- focused negative tests in `mcp_client_stdio_tool_call`
- gateway sent-frame inspection for redacted local args

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call`
- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_gateway_stdio_tool_call`

## Completion Signal

M44-03 is complete when negative tests lock in redacted diagnostics and local argument handling.
