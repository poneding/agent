# MX01-04 Desktop Bridge MCP Management Snapshot Spec

## Purpose

Provide UI-safe MCP management snapshot contracts for Desktop and bridge surfaces without building a full Desktop management page.

## Scope

This slice delivers:

- MCP management server snapshot DTOs in `ruse-mcp`
- bridge-consumable MCP snapshot DTOs in `ruse-ui-bridge`
- server status, transport kind, roots, sampling availability, stream status, auth status, last diagnostics, and extension health fields
- bounded/redacted serialization tests

Not included:

- full Desktop page
- live account picker UX
- token vault UX

## Architecture

`ruse-mcp` produces protocol-aware snapshots. `ruse-ui-bridge` defines bridge DTOs that can be populated by callers without importing MCP internals into bridge normalization paths.

## Security

Snapshots contain status metadata only. They do not include raw URLs with credentials, headers, bearer tokens, refresh tokens, secret ref names, prompts, tool arguments, resource contents, or remote failure bodies.

## Acceptance Criteria

- `cargo test -p ruse-mcp mcp_management_snapshot`
- `cargo test -p ruse-ui-bridge mcp_management_snapshot`
