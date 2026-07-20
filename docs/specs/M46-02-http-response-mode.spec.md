# M46-02 HTTP Response Mode Spec

## Purpose

Represent MCP HTTP responses as either a single JSON body or a bounded SSE frame list.

## Product Context

The transport and client need a common typed boundary before reqwest can classify `application/json` versus `text/event-stream` responses.

## Scope

This spec delivers:

- `McpClientHttpResponseMode`
- SSE event data to `McpClientHttpResponseFrame` extraction
- matching-id lookup helpers
- redacted malformed SSE JSON diagnostics

Owned by other specs:

- raw SSE parsing in M46-01
- reqwest response classification in M46-03
- client dispatch in M46-04

## Architecture

`client_http.rs` maps parsed SSE data into the existing JSON-RPC response frame contract.

## Public Contracts

```rust
pub enum McpClientHttpResponseMode {
    Json(McpClientHttpResponse),
    Sse(McpClientHttpSseResponse),
}
```

## State and Failure Handling

- JSON mode preserves existing parsing.
- SSE mode parses every event with non-empty data as a JSON-RPC response candidate.
- Non-JSON or non-response SSE data is rejected in M46 unless later specs define notifications.
- Matching-id lookup returns a safe unexpected-id error.

## Security and Privacy

- Raw SSE data is never included in public errors.
- Remote JSON-RPC failure messages remain redacted.

## Testing Strategy

- JSON mode compatibility
- SSE success/failure frame extraction
- malformed SSE JSON redaction
- missing matching id behavior

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_response_mode`
- focused clippy for `ruse-mcp`

## Completion Signal

M46-02 is complete when response mode contracts can extract existing JSON-RPC response frames from JSON and SSE inputs.
