# M43-05 Hardening Docs Spec

## Purpose

Document MCP client stdio transport behavior, verify focused M43 tests, and mark M43 complete only after final quality gates pass.

## Product Context

M43 introduces live external process IO for the MCP client side. Documentation must clearly distinguish implemented initialize/tools/list transport from later tool execution and reentrant callback work.

## Scope

This spec delivers:

- README documentation for MCP client stdio transport
- milestone status updates for M43 implementation
- final verification evidence in the M43 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- gateway tools/call over live stdio
- streamable HTTP client docs
- Desktop MCP server management UX

## Security and Privacy

- Docs must state that tool listing is discovery only, not permission to execute.
- Docs must state that errors keep process diagnostics redacted.
- Docs must state that secret refs are resolved outside `ruse-mcp`.

## Testing Strategy

- focused M43 tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_codec`
- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_session`
- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_process`
- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_mcp_client`
- `CARGO_TARGET_DIR=target/codex-m43 cargo xtask ci`

## Completion Signal

M43-05 is complete when docs describe implemented client stdio behavior, all focused tests and final gates pass, and M43 is marked complete in the milestone index.
