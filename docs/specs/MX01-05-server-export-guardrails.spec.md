# MX01-05 Server Export Expansion Guardrails Spec

## Purpose

Harden MCP server export manifests so future expansion cannot accidentally expose dangerous side-effecting local tools.

## Scope

This slice delivers:

- manifest capability declarations for tool exports
- side-effecting export guardrails
- policy preview contracts
- default rejection for command, edit, git-write, browser, secret, and network side-effecting exports
- tests for manifest validation and previews

Not included:

- enabling dangerous remote execution by default
- granting MCP servers local filesystem, command, Git, browser, Desktop, secret, or network permissions

## Architecture

`agent-mcp::server_manifest` owns manifest schema validation and preview contracts. Existing read-only workspace tool exports continue through the existing catalog path.

## Security

Enabled side-effecting export declarations fail validation before runtime profile construction. Read-only tools remain explicit and policy-gated through the existing MCP server export path.

## Acceptance Criteria

- `cargo test -p agent-mcp mcp_server_export_guardrails`
