# M04-05 Replay Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned replay fixtures and compatibility checks for M04 traces.

**Architecture:** `agent-storage` owns fixture loading and compatibility diagnostics. `agent-core` verifies recovered trace summaries from loaded fixtures.

**Tech Stack:** Rust 2024 edition, `agent-storage`, `agent-core`, `agent-types`, `serde_json`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Compatibility Tests

**Files:**
- Create: `crates/agent-storage/tests/replay_compatibility.rs`
- Create: `crates/agent-storage/tests/fixtures/m04_trace_v1.json`
- Create: `crates/agent-core/tests/recovery_compatibility.rs`

- [ ] **Step 1: Add failing fixture tests**

Tests must prove fixture JSON parsing, old trace replay, checkpoint fixture replay, unsupported future version diagnostics, and final summary reconstruction.

- [ ] **Step 2: Verify RED**

```bash
cargo test --workspace replay_compatibility
```

Expected: tests fail because compatibility loader contracts do not exist.

### Task 2: Implement Compatibility Loader

**Files:**
- Create: `crates/agent-storage/src/compatibility.rs`
- Modify: `crates/agent-storage/src/lib.rs`

- [ ] **Step 1: Define `ReplayCompatibilityReport`**

Use the fields from `docs/specs/M04-05-replay-compatibility.spec.md`.

- [ ] **Step 2: Implement fixture parsing**

Parse fixture schema version, runtime events, checkpoints, and expected summary.

- [ ] **Step 3: Verify storage GREEN**

```bash
cargo test -p agent-storage replay_compatibility
```

Expected: storage compatibility tests pass.

### Task 3: Verify Core Recovery Compatibility

**Files:**
- Modify: `crates/agent-core/tests/recovery_compatibility.rs`

- [ ] **Step 1: Reconstruct outcome from fixture**

Use loaded fixture events to rebuild final summary and audit count.

- [ ] **Step 2: Verify core GREEN**

```bash
cargo test -p agent-core recovery_compatibility
```

Expected: core compatibility tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M04-05-replay-compatibility.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Mark M04 complete**

Update milestone status so M04 is complete and active focus advances to `M05-context-memory`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m04 replay compatibility"
```
