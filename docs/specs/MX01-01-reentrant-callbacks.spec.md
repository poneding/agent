# MX01-01 Reentrant Client Callbacks Spec

## Purpose

Add typed contracts for MCP server-initiated callbacks that can arrive while a stdio or HTTP tool call is in flight. The slice covers safe fixture dispatch and reports for `roots/list`, `sampling/createMessage`, unknown callbacks, and policy-safe refusal.

## Scope

This slice delivers:

- callback transport/phase/outcome enums
- callback frame parsing for JSON-RPC request frames
- safe fixture dispatch for `roots/list` and `sampling/createMessage`
- unknown callback rejection
- redacted diagnostics and trace-safe callback reports
- fixture tests

Not included:

- changing live stdio or HTTP clients into fully concurrent multiplexers
- granting callbacks local tool, filesystem, command, Git, browser, Desktop, or secret permissions

## Architecture

`agent-mcp::client_callbacks` owns callback parsing and report rendering. It composes the existing root provider and sampling provider traits. Sampling callbacks reuse existing safe validation before provider execution.

## Security

Callback reports never expose raw sampling prompt text, provider payloads, local tool arguments, headers, tokens, or raw remote errors. Unknown or unsafe methods are rejected before touching providers.

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_client_reentrant_callbacks`
