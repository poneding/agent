# ADR-0004: Use Tokio for Async Runtime Paths

## Status

Accepted

## Context

The runtime needs streaming model output, concurrent tool execution, cancellable background tasks, MCP/plugin calls, shell process management, and UI event fan-out.

## Decision

Use Tokio for async runtime paths and structured concurrency.

## Consequences

Positive outcomes:

- Mature async ecosystem.
- Good process, networking, channel, and timeout primitives.
- Works across CLI and Desktop.
- Fits streaming model and tool output.

Tradeoffs:

- Async boundaries must be explicit.
- Detached tasks can create recovery and shutdown bugs.
- Tests need deterministic timeout and cancellation patterns.

## Alternatives Considered

| Alternative | Reason Not Chosen |
|---|---|
| blocking runtime only | poor fit for streaming and concurrent tasks |
| async-std | smaller ecosystem for this use case |
| custom executor | unnecessary complexity |

## Follow-Up

- Define cancellation and timeout rules in `docs/architecture/async-runtime.md`.
- Keep runtime startup in application crates.
- Avoid detached tasks in library crates.
