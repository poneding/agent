# MX01-02 MCP Progress Logging Task Events Spec

## Purpose

Represent MCP progress, logging, task, and elicitation protocol events as typed, redacted reports that CLI, Desktop, or bridge layers can consume without exposing raw protocol payloads.

## Scope

This slice delivers:

- protocol event kind/status contracts
- safe parsing from JSON-RPC notification/request shapes
- normalization of known progress/logging/task/elicitation methods
- bounded UI-safe summaries
- redacted diagnostics
- tests for secret-like payloads and unknown events

Not included:

- full product UI
- durable task execution semantics for remote MCP tasks
- exposing raw prompts, provider payloads, headers, tokens, or remote request bodies

## Architecture

`agent-mcp::client_events` owns protocol event reports and redaction. The output is serializable and can be passed through CLI JSON or bridge frames later.

## Security

Reports include method, event kind, counts, bounded status strings, and redacted messages only. Raw prompt, token, secret, header, and provider payload fields are intentionally not retained.

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_client_protocol_events`
