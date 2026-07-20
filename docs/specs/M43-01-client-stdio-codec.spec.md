# M43-01 Client Stdio Codec Spec

## Purpose

Add typed client-side JSON-RPC frame helpers for MCP stdio requests and responses.

## Product Context

Before a live process transport exists, `ruse-mcp` needs deterministic request rendering and response parsing contracts that can be tested without subprocess IO.

## Scope

This spec delivers:

- client request frame rendering
- response frame parsing for success and failure
- request id preservation and matching helpers
- bounded malformed frame errors

Owned by other specs:

- session state machine
- process transport
- typed `McpClient` operations

## Architecture

Add a new `ruse-mcp` module, likely `client_stdio`, that reuses existing `McpJsonRpcId`, `McpJsonRpcRequest`, `McpJsonRpcSuccess`, `McpJsonRpcFailure`, and `McpJsonRpcError`.

## Public Contracts

Expected contracts:

```rust
pub fn render_mcp_client_stdio_request_frame(
    id: McpJsonRpcId,
    method: impl Into<String>,
    params: serde_json::Value,
) -> Result<String, McpError>;

pub fn parse_mcp_client_stdio_response_frame(
    frame: &str,
) -> Result<McpClientStdioResponseFrame, McpError>;

pub enum McpClientStdioResponseFrame {
    Success(McpJsonRpcSuccess),
    Failure(McpJsonRpcFailure),
}
```

## State and Failure Handling

- Rendered request frames end in `\n`.
- Empty methods, oversized frames, malformed JSON, non-`2.0` JSON-RPC, missing ids, and frames that are neither success nor failure are rejected.
- Failure frames remain typed so callers can map server errors safely.

## Security and Privacy

- Error messages are bounded and redacted through existing MCP conventions.
- Request params are serialized only into the outgoing protocol frame, not into diagnostics.

## Testing Strategy

- render initialize request frame
- parse success response frame
- parse failure response frame
- reject notification or malformed response frames

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_codec`

## Completion Signal

M43-01 is complete when request/response frame helpers are public, exported, and covered by focused tests.
