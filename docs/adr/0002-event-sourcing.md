# ADR-0002: Use Event Sourcing for Runtime Facts

## Status

Accepted

## Context

Agent behavior must be auditable, replayable, recoverable, and visible to multiple UI surfaces. A turn can include model streams, permission decisions, tool calls, memory writes, checkpoints, and failures.

## Decision

Represent meaningful runtime facts as append-only typed events. Derived state, UI views, checkpoints, audit trails, and replay flows are built from these events.

## Consequences

Positive outcomes:

- Strong audit trail.
- Deterministic replay tests.
- Crash recovery from persisted events.
- UI clients can observe the same runtime truth.

Tradeoffs:

- Event schema design must be careful.
- Storage and migration requirements appear early.
- Sensitive data must be actively redacted before events are written.

## Alternatives Considered

| Alternative | Reason Not Chosen |
|---|---|
| mutable conversation row only | weak replay and audit behavior |
| log text transcripts only | hard to recover structured state |
| UI-driven state | would couple runtime correctness to presentation |

## Follow-Up

- Implement event contracts in `agent-types`.
- Keep storage sequencing in `agent-storage`.
- Add replay tests for every event-sourced milestone.
