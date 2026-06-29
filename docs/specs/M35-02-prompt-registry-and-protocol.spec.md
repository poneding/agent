# M35-02 Prompt Registry and Protocol Spec

## Purpose

Add an MCP prompt export registry and JSON-RPC support for `prompts/list` and `prompts/get`.

## Product Context

MCP clients expect prompt discovery and retrieval through standard protocol methods. The local server should expose only explicitly registered prompt exports.

## Scope

This spec delivers:

- `McpPromptExportRegistry`
- enabled/disabled prompt visibility
- `McpServerMethod::PromptsList`
- `McpServerMethod::PromptsGet`
- stdio frame handling for prompt methods

Owned by other specs:

- manifest prompt declarations
- CLI profile wiring

## Architecture

`agent-mcp::server_stdio` routes prompt methods to a prompt registry owned by the fixture/session.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m35 cargo test -p agent-mcp mcp_server_prompt_protocol`

## Completion Signal

M35-02 is complete when stdio sessions can list and get registered prompt exports.
