# ADR-0001: Use a Rust Workspace

## Status

Accepted

## Context

`agent` needs a fast, local-first runtime with strong type boundaries, reliable concurrency, low-level process control, and a maintainable crate structure.

## Decision

Use a Rust workspace with focused crates for runtime core, model adapters, tools, policy, sandboxing, storage, memory, scheduler, plugins, MCP, UI bridge, CLI, and Desktop.

## Consequences

Positive outcomes:

- Strong compile-time boundaries.
- Good fit for local process, filesystem, and sandbox work.
- Clear path to high-performance streaming and concurrent execution.
- Workspace crates can evolve independently while sharing typed contracts.

Tradeoffs:

- More upfront crate boundary design.
- More explicit async and error handling.
- Slower iteration than a single script during the earliest prototype.

## Alternatives Considered

| Alternative | Reason Not Chosen |
|---|---|
| single Rust crate | would blur runtime, tool, storage, and UI ownership |
| TypeScript runtime | easier UI integration but weaker local systems boundary |
| Python runtime | fast prototyping but weaker compile-time contracts for long-term runtime evolution |

## Follow-Up

- Keep crate boundaries documented in `docs/architecture/crate-boundaries.md`.
- Add crates only when ownership or testability justifies the split.
