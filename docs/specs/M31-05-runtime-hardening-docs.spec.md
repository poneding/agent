# M31-05 Runtime Hardening Docs Spec

## Purpose

Close M31 with lifecycle hardening, README documentation, milestone status, and final quality gates.

## Product Context

After the stdio server command exists, users need clear usage instructions and maintainers need evidence that the command is safe for scripts and MCP clients.

## Scope

This spec delivers:

- README usage docs for MCP stdio server
- milestone completion status
- final quality gate evidence
- command safety notes and known limits

Owned by other specs:

- session runtime: M31-01
- export profiles: M31-02
- CLI command: M31-03
- trace/audit: M31-04

## Architecture

Documentation references shared `agent-mcp` runtime contracts and CLI command surfaces. No new runtime ownership is introduced in this slice unless final hardening tests reveal a narrow fix.

## File Layout

```text
README.md
docs/milestones/README.md
docs/plans/M31-mcp-stdio-server-runtime.plan.md
```

## Public Contracts

```bash
agent-cli mcp server stdio
```

## State and Failure Handling

- docs describe EOF shutdown
- docs describe default empty export profile
- docs describe stdout/stderr separation
- final plan records verification commands

## Security and Privacy

- docs state that exports are explicit and policy-gated
- docs state that stdout is protocol-only
- docs state that diagnostics are redacted

## Testing Strategy

- final focused M31 tests
- full workspace checks
- `cargo xtask ci`

## Acceptance Criteria

- README documents the stdio server command.
- M31 milestone status is complete.
- final quality gate passes.

## Completion Signal

M31 is complete when docs, milestone status, and final quality gate evidence are committed.
