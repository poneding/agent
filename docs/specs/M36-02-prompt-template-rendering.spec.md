# M36-02 Prompt Template Rendering Spec

## Purpose

Render explicit prompt message templates from validated prompt arguments.

## Scope

This spec delivers:

- string-only argument validation
- deterministic `{{argument_name}}` substitution
- missing and unknown argument errors
- undeclared template reference validation
- redaction and bounding for rendered output

## Architecture

`agent-mcp::server_prompt` owns rendering inside `McpPromptExport`. The registry passes `prompts/get` arguments into the export and receives an `McpPromptGetResult`.

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m36 cargo test -p agent-mcp mcp_server_prompt_rendering`

## Completion Signal

M36-02 is complete when prompt exports render declared arguments deterministically and reject invalid argument payloads.
