# M66-03 MCP Management and Compatibility Health Spec

## Purpose

Productize the first MCP management health layer after plugin/skill hardening. M66-03 must improve user-visible MCP server diagnostics without expanding protocol scope or starting OAuth work.

## Scope

This slice delivers:

- a typed MCP server status health report derived from existing status summaries
- stable diagnostics for not-initialized, no-export, and capability/export mismatch states
- stable next-action ids for initialize, configure exports, and fix capabilities
- additive CLI JSON/text rendering through `mcp server status`
- focused tests proving redaction, stable JSON shape, and compatibility diagnostics
- milestone and roadmap status updates

Out of scope:

- OAuth completion
- live MCP server startup in status command
- new MCP protocol methods
- Desktop redesign
- broad support bundle integration

## Contract Requirements

The health report must be:

- typed and serializable
- additive to existing `McpServerStatusSummary`
- deterministic and credential-free
- bounded and redacted
- safe for CLI/JSON and future bridge/support evidence

Diagnostic codes:

- `mcp.server.not_initialized`
- `mcp.server.no_exports`
- `mcp.server.tool_exports_without_capability`
- `mcp.server.resource_exports_without_capability`

Next-action ids:

- `mcp.server.initialize`
- `mcp.server.configure_exports`
- `mcp.server.fix_capabilities`

## Acceptance Criteria

- `agent-mcp` exposes a typed status health report.
- `agent-cli mcp server status --json` includes the additive health report.
- Text status includes a concise health line.
- Capability/export mismatch produces an error diagnostic and unhealthy report status.
- Empty exports produce a warning diagnostic and configure-exports action.
- Not-initialized state produces an info diagnostic and initialize action.
- Diagnostics are redacted and bounded.

## Verification

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-mcp mcp_server_status_health
CARGO_TARGET_DIR=target/m66-dev cargo test -p agent-cli --test mcp_server_commands
CARGO_TARGET_DIR=target/m66-dev cargo clippy -p agent-mcp -p agent-cli --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```
