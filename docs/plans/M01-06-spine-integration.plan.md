# M01-06 Spine Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run an end-to-end fake turn, persist emitted events, reload them, and verify the replayed final state.

**Architecture:** This slice proves the M01 runtime spine works across `agent-core`, `agent-model`, `agent-storage`, and `agent-types`. It uses the fake provider, in-memory store, and replay helpers to verify that event-sourced execution can be reconstructed.

**Tech Stack:** Rust 2024 edition, M01 runtime crates, `tokio` async tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add Integration Test Fixture

**Files:**
- Create: `crates/agent-core/tests/spine_integration.rs`

- [x] **Step 1: Build a deterministic fake turn fixture**

The fixture should:

- create workspace/conversation/turn/correlation IDs for the fixture
- create `FakeModelProvider::new(FakeModelScript::text("hello from fake model"))`
- create `TurnInput`
- run `TurnEngine::run`
- append every emitted event to `InMemoryEventStore`

- [x] **Step 2: Verify persisted event stream**

Assertions:

- persisted event count is greater than zero
- all persisted events have `seq = Some`
- sequence values are contiguous from `1`
- at least one event has `RuntimeEventPayload::FinalResponseProduced`

Run:

```bash
cargo test -p agent-core spine_integration_persists_fake_turn_events
```

Expected: integration test passes.

### Task 2: Verify Replay Reconstructs Final Response

**Files:**
- Create: `crates/agent-storage/tests/replay_integration.rs`

- [x] **Step 1: Add replay final-state test**

The test should:

- load events from `InMemoryEventStore`
- iterate in sequence order
- collect final response text from `FinalResponseProduced`
- confirm it equals `hello from fake model`

Run:

```bash
cargo test -p agent-storage replay_reconstructs_final_response
```

Expected: replay reconstructs the same final response produced by the turn engine.

### Task 3: Add CLI Runtime Spine Smoke Test

**Files:**
- Create: `crates/agent-cli/tests/runtime_spine.rs`

- [x] **Step 1: Add CLI smoke assertion**

The test should run:

```bash
cargo run -p agent-cli
```

Expected stdout:

```text
agent (agent-model uses agent-types, agent-storage uses agent-types)
```

The CLI test proves the binary still links after runtime spine crates are integrated.

### Task 4: Run Full M01 Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [x] **Step 1: Run standard checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all commands pass.

- [x] **Step 2: Confirm M01 acceptance criteria**

Confirm:

```text
A fake turn runs from user input to final response.
Every meaningful state transition emits a typed event.
Persisted events replay into the same final state.
```

### Task 5: Commit and Advance

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M01 complete and set next focus**

Update the milestone index so M01 is complete and active focus moves to `M02-tool-policy-audit`.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m01 runtime spine"
```

Expected: commit succeeds.
