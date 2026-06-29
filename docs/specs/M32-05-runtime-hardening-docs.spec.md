# M32-05 Runtime Hardening Docs Spec

## Purpose

Close M32 with usage docs, milestone status, and final quality gate evidence for manifest-backed MCP stdio profiles.

## Product Context

Users and maintainers need a clear profile manifest example, safe startup notes, and proof that the profile runtime remains script-safe.

## Scope

This spec delivers:

- README manifest-backed stdio usage
- example manifest shape
- milestone status updates
- final quality gate evidence

Owned by other specs:

- manifest contracts: M32-01
- resource loader: M32-02
- tool catalog: M32-03
- CLI flags: M32-04

## Architecture

Documentation only unless final gates reveal narrow correctness issues.

## File Layout

```text
README.md
docs/milestones/README.md
docs/plans/M32-mcp-export-profile-runtime.plan.md
```

## Public Contracts

```bash
agent-cli mcp server stdio --profile path/to/mcp-server-exports.json --profile-name default
```

## State and Failure Handling

- docs describe default empty profile
- docs describe invalid manifest startup behavior
- docs describe stdout/stderr separation

## Security and Privacy

- docs state exports are explicit
- docs state policy is enforced on call/read
- docs state diagnostics and traces are redacted

## Testing Strategy

- final M32 focused tests
- `cargo xtask ci`

## Acceptance Criteria

- README documents profile manifests.
- M32 milestone status is complete.
- final quality gate passes.

## Completion Signal

M32 is complete when docs, status, and quality evidence are committed.
