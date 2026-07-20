# M35-03 Manifest Prompt Profile Spec

## Purpose

Extend MCP server export manifests with explicit prompt declarations.

## Scope

This spec delivers:

- prompt declaration schema under export profiles
- enabled/disabled prompt declarations
- profile runtime construction for prompt exports
- preflight inspection summary for prompt exports

## Architecture

`ruse-mcp::server_manifest` owns manifest parsing and converts prompt declarations into prompt export runtime entries.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m35 cargo test -p ruse-mcp mcp_server_manifest_prompts`

## Completion Signal

M35-03 is complete when manifest-backed prompt exports can be loaded into profile runtime state.
