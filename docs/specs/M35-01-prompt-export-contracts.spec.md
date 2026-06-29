# M35-01 Prompt Export Contracts Spec

## Purpose

Add typed MCP prompt export contracts in `agent-mcp`.

## Product Context

Prompt contracts are the durable runtime-owned shape used by MCP protocol handlers, CLI profile startup, and future Desktop MCP management views.

## Scope

This spec delivers:

- prompt descriptor contract
- prompt message contract
- prompt get/list result contracts
- validation, redaction, and bounds

Owned by other specs:

- JSON-RPC prompt method routing
- manifest prompt declarations
- CLI stdio startup wiring

## Architecture

`agent-mcp` owns MCP wire-safe prompt contracts. It must not depend on `agent-cli`.

## File Layout

```text
crates/agent-mcp/src/server_prompt.rs
crates/agent-mcp/src/lib.rs
crates/agent-mcp/tests/mcp_server_prompt_export.rs
```

## Public Contracts

- `McpPromptDescriptor`
- `McpPromptMessage`
- `McpPromptGetResult`
- `McpPromptListResult`
- `McpPromptExport`

## State and Failure Handling

- Empty prompt names are rejected.
- Secret-like names and descriptions are rejected.
- Secret-like message content is redacted in wire-safe output.
- Long message text is bounded.

## Security and Privacy

- Prompt content is explicit and bounded.
- Hidden prompt layers are not modeled here; callers supply already safe messages.
- Redaction is applied before serialization.

## Testing Strategy

- stable serialization tests
- invalid contract tests
- content redaction and bounds tests

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_export`

## Completion Signal

M35-01 is complete when prompt export contracts are public and tested.
