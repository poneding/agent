# M36-01 Prompt Argument Contracts Spec

## Purpose

Add typed MCP prompt argument descriptors and request parameter contracts.

## Scope

This spec delivers:

- `McpPromptArgument`
- descriptor serialization with optional `arguments`
- `McpPromptGetParams` for `prompts/get`
- contract tests for stable JSON shape

## Architecture

`ruse-mcp::server_prompt` owns the contracts and exports them through `ruse-mcp::lib`.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_prompt_arguments`

## Completion Signal

M36-01 is complete when prompt argument contracts are public, serializable, and tested.
