# Event Sourcing

The runtime records meaningful behavior as typed events. Events are the source of audit, trace replay, recovery, UI updates, and deterministic tests.

## Event Principles

- Events describe facts that happened, not commands that might happen.
- Events are append-only.
- Event order is stable within a conversation stream.
- Events carry actor, correlation, causation, timestamp, schema version, and payload.
- Events avoid provider-specific or UI-specific shapes.

## Event Envelope

The durable event envelope is defined in `agent-types` and consumed by storage, core, CLI, Desktop, tests, and replay.

Required fields:

- `schema_version`
- `id`
- `seq`
- `occurred_at`
- `workspace_id`
- `conversation_id`
- `turn_id`
- `task_id`
- `agent_id`
- `team_id`
- `actor`
- `correlation_id`
- `causation_id`
- `payload`

`seq` is assigned by storage. Runtime code may construct events with no sequence before persistence.

## Event Categories

| Category | Examples |
|---|---|
| Lifecycle | workspace opened, session started, conversation started |
| Turn | turn started, state changed, completed, failed |
| Model | stream started, output delta, stream completed |
| Tool | proposed, authorized, executed, observed |
| Policy | permission requested, decision recorded, lease issued |
| Memory | candidate proposed, record written, record deleted |
| Context | prompt assembled, compacted, pinned message preserved |
| Scheduler | task queued, leased, paused, resumed, completed |
| Replay | replay started, replay completed, replay mismatch |
| Diagnostic | warning, recoverable error, terminal error |

## Replay Contract

Replay rebuilds derived state from events. It must not repeat external side effects.

Replay rules:

- Replaying model deltas reconstructs model-visible output.
- Replaying tool result events reconstructs observations, not tool execution.
- Replaying permission decisions reconstructs audit state, not user prompts.
- Replaying checkpoint records can accelerate recovery but cannot contradict events.
- Replaying unknown future events requires a migration or version gate.

## Checkpoint Relationship

Events are authoritative. Checkpoints are derived snapshots used to resume faster.

Checkpoint rules:

- A checkpoint references the last included event sequence.
- A checkpoint contains enough runtime state to resume deterministic work.
- A checkpoint never replaces the event log.
- A checkpoint with a missing event range is invalid for recovery.

## Idempotency

Every side-effecting operation must carry an idempotency key before execution. Recovery uses the key to decide whether to reuse an existing result, reconcile external state, or stop for operator review.

## Testing Requirements

- Event JSON round trips preserve all fields.
- Event sequence ordering is deterministic.
- Replay from events reaches the expected state.
- Replay from checkpoint plus remaining events reaches the same state as full replay.
- Side-effecting events are not re-executed during replay.
