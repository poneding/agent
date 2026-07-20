# M36-04 CLI Stdio Prompt Arguments Spec

## Purpose

Serve manifest-backed prompt arguments through `ruse-cli mcp server stdio`.

## Scope

This spec delivers:

- `prompts/list` argument descriptors over CLI stdio
- `prompts/get` with `arguments` object over CLI stdio
- protocol-only stdout preservation
- invalid argument error e2e tests

## Architecture

`ruse-cli` passes manifest-backed prompt exports into `ruse-mcp`; it does not implement prompt rendering logic.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-cli mcp_server_prompt_arguments`

## Completion Signal

M36-04 is complete when the CLI stdio command renders manifest prompt arguments through MCP prompt methods.
