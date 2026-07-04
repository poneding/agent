# M35-04 CLI Stdio Prompt Profile Spec

## Purpose

Wire manifest-backed prompt exports into `agent-cli mcp server stdio`.

## Scope

This spec delivers:

- stdio profile startup with prompt exports
- prompt-capable server capabilities
- CLI JSON-RPC e2e tests for `prompts/list` and `prompts/get`
- protocol-only stdout preservation

## Architecture

`agent-cli` reads manifest files and passes prompt exports to `agent-mcp`; it does not implement prompt protocol logic.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-cli mcp_server_prompt_profile`

## Completion Signal

M35-04 is complete when the CLI stdio command serves manifest prompt exports through MCP prompt methods.
