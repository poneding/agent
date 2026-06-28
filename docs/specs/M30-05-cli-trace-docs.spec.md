# M30-05 CLI Trace Docs Spec

## Purpose

Expose MCP server readiness through safe CLI/trace surfaces and close M30 with documentation and final gates.

## Product Context

Users need to know whether local MCP server mode is available, what it would export, and where to inspect diagnostics. This slice makes server mode visible without requiring a production daemon.

## Scope

This spec delivers:

- CLI command shape for MCP server readiness
- UI-safe/trace-safe server export summaries
- README and milestone updates
- final M30 quality gate

Owned by other specs:

- protocol contracts: M30-01
- tool/resource export behavior: M30-02 and M30-03
- stdio fixture handling: M30-04

## Architecture

CLI renders summaries from `agent-mcp` contracts. Trace events use existing redaction and UI-safe summary patterns.

## Public Contracts

```bash
agent-cli mcp server status
agent-cli --json mcp server status
```

## State and Failure Handling

- CLI status succeeds without starting a server
- invalid command options return script-safe errors
- diagnostics do not expose raw resource contents or secrets

## Security and Privacy

- status output contains counts, names, and capability flags only
- no local tools execute during status rendering
- JSON output is parseable and redacted

## Testing Strategy

- CLI parse tests
- text and JSON render tests
- trace/summary redaction tests
- final `cargo xtask ci`

## Acceptance Criteria

- README documents MCP server export status.
- M30 milestone status is complete.
- final quality gate passes.

## Completion Signal

M30 is complete when CLI/docs expose server readiness safely, final checks pass, and milestone status is committed.
