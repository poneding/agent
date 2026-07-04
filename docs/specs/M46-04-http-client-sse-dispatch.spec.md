# M46-04 HTTP Client SSE Dispatch Spec

## Purpose

Integrate SSE response frames into `McpHttpClient` request dispatch.

## Product Context

Once the transport can return response modes, initialize, tools/list, and tools/call should work when a server answers with SSE frames instead of one JSON body.

## Scope

This spec delivers:

- `McpHttpClient` dispatch over response modes
- matching-id selection from SSE frames
- fixture tests for initialize, tools/list, and tools/call over SSE mode
- safe errors for missing matching response ids

Owned by other specs:

- parser contracts in M46-01
- response mode in M46-02
- reqwest transport in M46-03
- docs and final gates in M46-05

## Architecture

Existing `send_request` state flow remains intact; only the response parsing step changes from one body to response-mode extraction.

## Public Contracts

No CLI/Desktop contract change is expected. Existing `McpClient`, `McpClientRootProvider`, and `McpGatewayClient` implementations continue to work.

## State and Failure Handling

- Request id allocation remains deterministic.
- Initialize only records session state after matching response result maps successfully.
- Missing matching id fails before state mutation.
- Remote failure frames become bounded client errors.

## Security and Privacy

- SSE data and tool arguments remain excluded from diagnostics.
- Local policy still gates gateway tool execution before remote calls.

## Testing Strategy

- fixture initialize over SSE mode
- fixture tools/list over SSE mode
- gateway tools/call over SSE mode
- missing id and redacted failure regressions

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p agent-mcp mcp_client_http_sse_dispatch`
- existing M45 HTTP tests still pass

## Completion Signal

M46-04 is complete when existing HTTP client and gateway behavior works with SSE response frames.
