# M15-05 Persisted Turn Events Spec

## Purpose

Persist events emitted by a configured prompt turn and prove replay reconstructs the final response.

## Product Context

PRD acceptance requires complete turns to be replayable from stored events. M15-05 proves the new config-driven path preserves that runtime guarantee.

## Scope

This spec delivers:

- configured turn persistence helper
- in-memory event-store integration test
- SQLite event-store integration test if existing store APIs support it without extra setup
- CLI JSON prompt output as parseable UI event frames

Owned by later specs:

- resumable live turn checkpoints
- durable CLI session databases
- trace replay commands for configured prompt turns

## Architecture

Persistence remains owned by `agent-storage`. `agent-core::model_runtime` appends emitted turn events through the `EventStore` trait after successful or failed configured turns. Replay uses existing storage APIs.

## State And Failure Handling

- successful turn events are appended in emitted order
- failed turn events are appendable for diagnostics
- duplicate append errors return typed storage errors
- replay over persisted events reconstructs final response payload

## Security And Privacy

- persisted events contain final response and model deltas, not raw config files
- tests assert persisted JSON excludes secret-like fixture values

## Testing Strategy

- configured fake turn persists events in memory
- replay finds `FinalResponseProduced`
- CLI JSON prompt emits parseable `UiJsonFrame::Event` lines and `Done`
- JSON output excludes secret-looking values

## Acceptance Criteria

- Config-driven prompt turns preserve event-sourced replay guarantees.
- M15 can be marked complete.

## Completion Signal

M15-05 is complete when focused storage/CLI tests and workspace gates pass, M15 status is marked complete, and the next milestone focus is recorded.
