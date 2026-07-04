# M31-03 CLI Stdio Command Spec

## Purpose

Expose a script-safe `agent-cli mcp server stdio` command that runs the MCP server session over stdin/stdout.

## Product Context

MCP clients launch local servers as subprocesses. The CLI must provide a stable command that starts a session, keeps stdout protocol-only, and reports lifecycle diagnostics safely.

## Scope

This spec delivers:

- `mcp server stdio` CLI parsing
- stdin/stdout loop over `McpServerSession`
- text-free protocol stdout
- JSON/status-safe error handling

Owned by other specs:

- session runtime: M31-01
- export profiles: M31-02
- trace/audit summaries: M31-04

## Architecture

`agent-cli` owns IO handles and process exit codes. `agent-mcp` owns frame/session behavior. CLI tests use subprocess stdin/stdout fixtures.

## File Layout

```text
crates/agent-cli/src/command.rs
crates/agent-cli/tests/mcp_server_stdio_command.rs
```

## Public Contracts

```bash
agent-cli mcp server stdio
agent-cli mcp server stdio --max-frames 10
```

## State and Failure Handling

- EOF closes the session with exit code 0
- fatal IO errors exit non-zero with redacted stderr
- invalid CLI options return script-safe errors
- stdout contains only JSON-RPC response frames

## Security and Privacy

- no prompt text or diagnostics are written to stdout
- stderr diagnostics are redacted
- default command starts with empty export profile

## Testing Strategy

- CLI parse tests
- subprocess initialize/list smoke tests
- EOF shutdown tests
- invalid flag tests

## Acceptance Criteria

- `cargo test -p agent-cli mcp_server_stdio_command` passes.
- command can serve initialize and list frames over stdio.

## Completion Signal

M31-03 is complete when CLI stdio command parsing and subprocess tests are implemented.
