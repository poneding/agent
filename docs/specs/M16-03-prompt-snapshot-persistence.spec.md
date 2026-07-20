# M16-03 Prompt Snapshot Persistence Spec

## Purpose

Persist prompt profile and prompt snapshot metadata before model invocation so configured turns are auditable and replayable even when the model call fails.

## Product Context

M16-01 and M16-02 make prompt assembly structured and diagnosable. M16-03 moves that metadata into runtime events, giving trace replay and future Desktop/TUI surfaces a durable record of which prompt profile and layer hashes influenced a turn.

## Scope

This slice delivers:

- `PromptLayerHash`
- `PromptSnapshot`
- stable snapshot hashing in `ruse-context`
- `PromptSnapshotRecorded` runtime event payload in `ruse-types`
- configured turn integration in `ruse-core`
- persisted event tests proving snapshot metadata is written before model invocation

Owned by later slices:

- CLI inspection commands
- UI bridge prompt summary rendering
- encrypted artifact storage for raw hidden prompt content

## Architecture

`ruse-context` computes `PromptSnapshot` from `PromptBundle` and `PromptAssemblyReport`. `ruse-core` calls prompt assembly during configured turn preparation and emits a `PromptSnapshotRecorded` event before the turn invokes a model provider. `ruse-storage` persists the event through existing event store APIs without schema branching.

Snapshot payloads store hashes and metadata, not hidden prompt text.

## Public Contracts

```rust
pub struct PromptLayerHash {
    pub layer_id: PromptLayerId,
    pub source_hash: String,
    pub content_hash: String,
}

pub struct PromptSnapshot {
    pub profile_id: PromptProfileId,
    pub profile_version: u32,
    pub layer_hashes: Vec<PromptLayerHash>,
    pub bundle_hash: String,
    pub diagnostics: Vec<PromptDiagnostic>,
    pub message_count: u32,
    pub estimated_tokens: u32,
}

pub struct PromptSnapshotRecorded {
    pub profile_id: String,
    pub profile_version: u32,
    pub bundle_hash: String,
    pub layer_hashes: Vec<PromptLayerHashRecord>,
    pub diagnostic_count: u32,
    pub message_count: u32,
    pub estimated_tokens: u32,
}
```

## State And Failure Handling

- Snapshot hashing must be stable for identical profile, layer, report, and bundle data.
- Prompt assembly failures fail the turn before model invocation.
- Snapshot event append failures fail the configured turn before model invocation.
- Model failures still persist the already-recorded prompt snapshot event.

## Security And Privacy

Snapshot events do not include raw prompt content. Hash inputs may include content internally, but event payloads expose only layer ids, source labels or hashes, content hashes, bundle hash, counts, and diagnostic metadata. Secret-like values must not appear in event JSON.

## Testing Strategy

- snapshot hash stability tests in `ruse-context`
- snapshot serialization excludes raw layer content
- runtime event JSON shape test for `PromptSnapshotRecorded`
- configured turn emits prompt snapshot before model stream events
- configured turn persists prompt snapshot on model failure
- replay from persisted events can find profile id and bundle hash

## Acceptance Criteria

- Every configured turn records prompt profile id, profile version, bundle hash, layer hashes, and counts before model invocation.
- Snapshot payloads are UI-safe and replay-safe.
- Model failure does not erase prompt auditability.

## Completion Signal

M16-03 is complete when focused context, types, and core persistence tests pass, workspace gates pass, and active implementation advances to M16-04.
