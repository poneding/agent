# Testing Strategy

Testing must prove runtime behavior, not just compile paths.

## Test Layers

| Layer | Purpose |
|---|---|
| unit | pure functions, state transitions, serialization |
| integration | crate interactions with fake providers and local stores |
| end-to-end | CLI/runtime flows with deterministic fixtures |
| replay | event logs rebuild the same state |
| property | path matching, policy decisions, budget allocation |
| fuzz | manifests, MCP messages, tool args, event JSON |
| concurrency | leases, cancellation, locks, scheduler limits |
| snapshot | JSON contracts, trace output, CLI output |

## Test Fixtures

Fixtures live near the crate that owns the behavior. Shared protocol fixtures can live under `tests/fixtures/` once the workspace exists.

Fixture rules:

- deterministic data
- no real secrets
- no network dependency
- stable timestamps
- stable UUIDs
- explicit schema version

## Required Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

When available:

```bash
cargo nextest run --workspace
cargo test --workspace --doc
```

## Replay Tests

Replay tests are mandatory for event-sourced behavior.

They prove:

- stored event JSON deserializes
- event order is stable
- derived state matches expected state
- checkpoint plus tail replay matches full replay
- side effects are not repeated

## Security Tests

Security tests cover:

- denied permissions
- approval-required states
- sandbox path boundaries
- command allowlist behavior
- secret redaction
- plugin capability validation
- MCP tool policy mapping
