# M04-03 Recovery Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resume interrupted work from the latest checkpoint plus event-log tail.

**Architecture:** `agent-core` owns `RecoveryPlan` and `RecoveryEngine`. It consumes `EventStore`, `CheckpointStore`, and `IdempotencyStore` without adding UI dependencies.

**Tech Stack:** Rust 2024 edition, `agent-core`, `agent-storage`, `agent-types`, `serde_json`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Recovery Tests

**Files:**
- Create: `crates/agent-core/tests/recovery_flow.rs`

- [ ] **Step 1: Add failing tests**

Tests must prove planning without checkpoint, planning from latest checkpoint, replay from checkpoint tail, idempotency-aware resume, and corrupt checkpoint diagnostics.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-core recovery_flow
```

Expected: tests fail because recovery contracts do not exist.

### Task 2: Implement Recovery Contracts

**Files:**
- Create: `crates/agent-core/src/recovery.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Define `RecoveryPlan` and `RecoveryOutcome`**

Use the fields from `docs/specs/M04-03-recovery-flow.spec.md`.

- [ ] **Step 2: Implement planner**

Select the latest checkpoint for the conversation/turn and compute the replay cursor.

- [ ] **Step 3: Implement recovery runner**

Replay events from the cursor and reuse recorded idempotent side effects.

- [ ] **Step 4: Verify GREEN**

```bash
cargo test -p agent-core recovery_flow
```

Expected: focused recovery tests pass.

### Task 3: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M04-03-recovery-flow.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Advance active focus**

Update active focus from `M04-03-recovery-flow` to `M04-04-trace-commands`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m04-03 recovery flow"
```
