# M15-05 Persisted Turn Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist config-driven turn events and emit JSON prompt frames that replay cleanly.

**Architecture:** Extend `agent-core::model_runtime` with event-store persistence helpers. CLI JSON prompt output normalizes emitted runtime events through `agent-ui-bridge`.

**Tech Stack:** Rust 2024 edition, existing `EventStore`, in-memory storage, UI bridge frames, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-05-persisted-turn-events.spec.md`
- Read: `crates/agent-core/src/model_runtime.rs`
- Read: `crates/agent-storage/src/lib.rs`
- Read: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M15-05-persisted-turn-events.spec.md
```

Expected: the spec defines persistence, replay, JSON output, and privacy requirements.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/configured_turn_persistence.rs`
- Modify: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `configured_turn_persistence_appends_events_in_order`
- `configured_turn_persistence_replays_final_response`
- `json_mode_prompt_runtime_emits_event_frames`
- `json_mode_prompt_runtime_excludes_secret_like_values`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core --test configured_turn_persistence
cargo test -p agent-cli --test json_mode json_mode_prompt_runtime
```

Expected: tests fail because persistence and JSON prompt frames are not implemented for configured prompt turns.

### Task 3: Implement Persistence And JSON Prompt Frames

**Files:**
- Modify: `crates/agent-core/src/model_runtime.rs`
- Modify: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Add persistence helper**

Append emitted configured-turn events to an `EventStore` and return the configured turn outcome.

- [x] **Step 2: Render JSON prompt events**

For `--json --prompt`, emit normalized `UiJsonFrame::Event` lines followed by `Done` with the event count.

- [x] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-core --test configured_turn_persistence
cargo test -p agent-cli --test json_mode json_mode_prompt_runtime
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-05-persisted-turn-events.plan.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m15 persisted turn events"
```

Expected: commit succeeds.
