# M45-05 Hardening Docs Spec

## Purpose

Document MCP streamable HTTP client behavior, verify focused M45 tests, and mark M45 complete only after final quality gates pass.

## Product Context

M45 adds HTTP MCP client support beside stdio. Documentation must distinguish implemented request/response HTTP operations from later streaming, auth, progress, and Desktop management work.

## Scope

This spec delivers:

- README documentation for HTTP MCP client discovery and tool execution
- milestone status updates for M45 implementation
- final verification evidence in the M45 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- SSE streaming and resumable streams
- OAuth/DCR and token refresh
- progress/logging/task protocols
- Desktop MCP management UX

## Security and Privacy

- Docs must state local policy gates gateway tool execution.
- Docs must state HTTP diagnostics are redacted.
- Docs must state remote success output is returned data and then locally bounded.

## Testing Strategy

- focused M45 tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_contracts`
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_mcp_client`
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_gateway_http_tool_call`
- `CARGO_TARGET_DIR=target/codex-m45 cargo xtask ci`

## Completion Signal

M45-05 is complete when docs describe implemented HTTP behavior, all focused tests and final gates pass, and M45 is marked complete in the milestone index.
