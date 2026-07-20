# M46-05 Hardening Docs Spec

## Purpose

Document MCP HTTP SSE response behavior, verify focused M46 tests, and mark M46 complete only after final quality gates pass.

## Product Context

M46 adds bounded SSE response parsing beside M45's JSON response path. Documentation must distinguish response-stream parsing from later resumable/incremental stream lifecycle work.

## Scope

This spec delivers:

- README documentation for HTTP SSE response support
- milestone status updates for M46 implementation
- final verification evidence in the M46 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- resumable streams
- OAuth/DCR and token refresh
- progress/logging/task protocols
- Desktop MCP management UX

## Security and Privacy

- Docs must state local policy still gates gateway tool execution.
- Docs must state SSE diagnostics are bounded and redacted.
- Docs must state M46 parses bounded response bodies, not long-lived resumable streams.

## Testing Strategy

- focused M46 tests
- M45 HTTP regression tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_contracts`
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_response_mode`
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_transport`
- `CARGO_TARGET_DIR=target/codex-m46 cargo test -p ruse-mcp mcp_client_http_sse_dispatch`
- `CARGO_TARGET_DIR=target/codex-m46 cargo xtask ci`

## Completion Signal

M46-05 is complete when docs describe implemented SSE response behavior, all focused tests and final gates pass, and M46 is marked complete in the milestone index.
