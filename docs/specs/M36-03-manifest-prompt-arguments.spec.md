# M36-03 Manifest Prompt Arguments Spec

## Purpose

Extend MCP server export manifests with prompt argument declarations.

## Scope

This spec delivers:

- manifest prompt `arguments` schema
- profile runtime construction with argument descriptors
- inspection summary for prompt arguments without prompt message text
- duplicate and invalid argument diagnostics

## Architecture

`ruse-mcp::server_manifest` parses argument declarations and builds `McpPromptExport` values.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m36 cargo test -p ruse-mcp mcp_server_manifest_prompt_arguments`

## Completion Signal

M36-03 is complete when manifest-backed prompt exports can declare arguments and profile inspection summarizes them safely.
