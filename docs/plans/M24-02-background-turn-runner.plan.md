# M24-02 Background Turn Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run queued background tasks through shared comprehensive turn execution using scheduler leases.

**Architecture:** `agent-core` owns `BackgroundTaskRunner`; `agent-scheduler` owns queues and leases; storage and model/tool behavior remain behind existing core abstractions.

**Tech Stack:** Rust 2024 edition, Tokio-friendly runner contracts, fake model fixtures, scheduler leases, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-02-background-turn-runner.spec.md`
- Read: `crates/agent-core/src/tool_loop.rs`
- Read: `crates/agent-core/src/model_runtime.rs`
- Read: `crates/agent-scheduler/src/queue.rs`

- [ ] **Step 1: Review runner scope**

Run:

```bash
sed -n '1,260p' docs/specs/M24-02-background-turn-runner.spec.md
```

Expected: runner owns lease-to-turn orchestration and not task command mutation.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/background_turn_runner.rs`

- [ ] **Step 1: Add no-ready-task test**

Runner returns `Ok(None)` when the queue has no ready tasks.

- [ ] **Step 2: Add successful turn test**

Queued task is leased, executed through fake comprehensive turn, completed, and summarized.

- [ ] **Step 3: Add failure test**

Model/tool failure marks lease failed and preserves safe diagnostics.

- [ ] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-core background_turn_runner
```

Expected: tests fail because background runner contracts are missing.

### Task 3: Implement Runner

**Files:**
- Create: `crates/agent-core/src/background.rs`
- Modify: `crates/agent-core/src/lib.rs`
- Modify if needed: `crates/agent-scheduler/src/queue.rs`

- [ ] **Step 1: Add runner contracts**

Implement input, outcome, worker validation, and fixture request mapping.

- [ ] **Step 2: Lease and execute one task**

Acquire one ready lease, build turn input, consume model stream, and record event count.

- [ ] **Step 3: Complete or fail lease**

Map successful final response to complete and model/tool errors to fail.

- [ ] **Step 4: Preserve replay-safe summaries**

Summaries are bounded and redacted.

### Task 4: Verify

- [ ] **Step 1: Run focused tests**

```bash
cargo test -p agent-core background_turn_runner
```

- [ ] **Step 2: Run workspace checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Task 5: Update Status and Commit

- [ ] **Step 1: Mark M24-02 complete**

Advance focus to M24-03.

- [ ] **Step 2: Commit M24-02**

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add background turn runner"
```
