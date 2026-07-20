# M16-03 Prompt Snapshot Persistence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist prompt snapshot metadata before configured turns invoke model providers.

**Architecture:** `ruse-context` computes snapshot hashes. `ruse-types` defines the runtime event payload. `ruse-core` emits and persists the prompt snapshot event before model invocation.

**Tech Stack:** Rust 2024 edition, serde, existing runtime events, existing event stores, deterministic fake model tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-03-prompt-snapshot-persistence.spec.md`
- Read: `crates/ruse-context/src/prompt.rs`
- Read: `crates/ruse-types/src/event.rs`
- Read: `crates/ruse-core/src/model_runtime.rs`
- Read: `crates/ruse-core/tests/configured_turn_persistence.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,240p' docs/specs/M16-03-prompt-snapshot-persistence.spec.md
```

Expected: the spec defines snapshot hashing, event payload, and configured turn persistence behavior.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-context/tests/prompt_snapshots.rs`
- Create: `crates/ruse-types/tests/m16_prompt_snapshot.rs`
- Modify: `crates/ruse-core/tests/configured_turn_persistence.rs`

- [x] **Step 1: Add context snapshot tests**

Add tests named:

- `prompt_snapshot_hash_is_stable_for_same_bundle`
- `prompt_snapshot_serialization_excludes_raw_content`

- [x] **Step 2: Add runtime event shape test**

Add test named:

- `m16_prompt_snapshot_event_serializes_stable_shape`

- [x] **Step 3: Add configured turn persistence tests**

Add tests named:

- `configured_turn_persistence_records_prompt_snapshot_before_model_stream`
- `configured_turn_persistence_keeps_prompt_snapshot_when_model_fails`

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p ruse-context --test prompt_snapshots
cargo test -p ruse-types --test m16_prompt_snapshot
cargo test -p ruse-core --test configured_turn_persistence
```

Expected: tests fail because prompt snapshots and snapshot runtime events are not implemented.

### Task 3: Implement Snapshot Persistence

**Files:**
- Modify: `crates/ruse-context/src/prompt.rs`
- Modify: `crates/ruse-context/src/lib.rs`
- Modify: `crates/ruse-types/src/event.rs`
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Add snapshot contracts and hashing**

Implement:

- `PromptLayerHash`
- `PromptSnapshot`
- stable bundle hash creation

- [x] **Step 2: Add runtime event payload**

Add `PromptSnapshotRecorded` to `RuntimeEventPayload`.

- [x] **Step 3: Emit snapshot event before model invocation**

Update configured turn execution to assemble the default prompt profile, build a snapshot, and prepend the snapshot event to the turn event stream before model stream events.

- [x] **Step 4: Preserve failure persistence**

Ensure model failures still return events containing the prompt snapshot.

### Task 4: Verify And Update Status

**Files:**
- Modify: `docs/plans/M16-03-prompt-snapshot-persistence.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-context --test prompt_snapshots
cargo test -p ruse-types --test m16_prompt_snapshot
cargo test -p ruse-core --test configured_turn_persistence
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

- [x] **Step 3: Mark M16-03 complete**

Update M16 planning status so active work advances to M16-04.

- [ ] **Step 4: Commit**

Run:

```bash
git add crates/ruse-context crates/ruse-types crates/ruse-core docs
git commit -m "feat: persist m16 prompt snapshots"
```

Expected: commit succeeds if this slice is landed independently.
