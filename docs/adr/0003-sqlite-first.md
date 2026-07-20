# ADR-0003: Use SQLite First for Local Persistence

## Status

Accepted

## Context

The initial product is local-first and must persist event logs, checkpoints, tasks, audit records, memory records, artifacts, and configuration without requiring a server.

## Decision

Use SQLite as the first durable storage backend behind `ruse-storage` traits.

## Consequences

Positive outcomes:

- No external service required.
- Good transactional behavior for local event streams.
- Portable across CLI and Desktop.
- Compatible with migration-based schema evolution.

Tradeoffs:

- Multi-user server deployments will need another backend.
- Long-running concurrent workloads need careful transaction and lock design.
- Large artifacts should be stored outside hot relational tables.

## Alternatives Considered

| Alternative | Reason Not Chosen |
|---|---|
| plain JSON files | weak concurrency and migration story |
| Postgres first | unnecessary local setup burden |
| embedded key-value store | less convenient relational querying for audit and traces |

## Follow-Up

- Keep storage traits backend-neutral.
- Store large artifacts by reference.
- Add migration fixtures before production hardening.
