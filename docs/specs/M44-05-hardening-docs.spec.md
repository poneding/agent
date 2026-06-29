# M44-05 Hardening Docs Spec

## Purpose

Document MCP stdio tool-call behavior, verify focused M44 tests, and mark M44 complete only after final quality gates pass.

## Product Context

M44 turns live MCP stdio discovery into executable remote tool calls. Documentation must distinguish implemented gateway `tools/call` support from later reentrant callbacks, progress, and streamable HTTP work.

## Scope

This spec delivers:

- README documentation for live MCP stdio tool execution
- milestone status updates for M44 implementation
- final verification evidence in the M44 plan
- acceptance criteria aligned to actual focused test names

Owned by later milestones:

- reentrant server-to-client callbacks
- streamable HTTP client docs
- Desktop MCP server management UX
- MCP progress/logging/task protocols

## Security and Privacy

- Docs must state local policy gates local gateway tool execution.
- Docs must state tool-call diagnostics are redacted.
- Docs must state remote success output is returned data and then locally bounded by gateway surfaces.

## Testing Strategy

- focused M44 tests
- `git diff --check`
- workspace fmt and `cargo xtask ci`

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call`
- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_gateway_stdio_tool_call`
- `CARGO_TARGET_DIR=target/codex-m44 cargo test -p agent-mcp mcp_client_stdio_tool_call_process`
- `CARGO_TARGET_DIR=target/codex-m44 cargo xtask ci`

## Completion Signal

M44-05 is complete when docs describe implemented tool-call behavior, all focused tests and final gates pass, and M44 is marked complete in the milestone index.
