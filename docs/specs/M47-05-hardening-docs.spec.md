# M47-05 Hardening Docs Spec

## Purpose

Document MCP HTTP session metadata behavior, verify focused M47 tests, and mark M47 complete only after final quality gates pass.

## Product Context

M47 adds resumability foundation without long-lived streams. Documentation must distinguish implemented session/checkpoint behavior from later reconnect, OAuth, progress, logging, and Desktop management work.

## Scope

This spec delivers:

- README documentation for HTTP session metadata and SSE checkpoints
- milestone status updates for M47 completion
- final focused test evidence
- final workspace CI evidence

Owned by later milestones:

- long-lived GET stream ownership
- reconnect and missed-event replay
- OAuth/DCR and token refresh
- progress/logging/task protocols
- Desktop stream management UX

## Architecture

No new runtime architecture is introduced. This slice verifies and documents the M47 runtime boundary.

## Public Contracts

Documentation names the public contracts implemented during M47 and states unsupported behaviors explicitly.

## State and Failure Handling

- Milestone status changes only after focused tests and final gate pass.
- Plans record verification evidence with dates and exact commands.
- README does not imply unsupported reconnect behavior.

## Security and Privacy

- Docs preserve the redaction story for session ids, event ids, raw headers, request bodies, and SSE data.
- Docs state that local policy still gates gateway tool execution.

## Testing Strategy

- run all M47 focused tests
- run final workspace gate
- run `git diff --check`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_metadata`
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_response_envelope`
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_session_headers`
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p ruse-mcp mcp_client_http_sse_checkpoints`
- `CARGO_TARGET_DIR=target/codex-m47 cargo xtask ci`

## Completion Signal

M47-05 is complete when docs and milestone status match implemented behavior and final M47 gates pass.
