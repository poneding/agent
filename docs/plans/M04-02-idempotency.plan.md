# M04-02 Idempotency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record completed side-effecting tool calls and return recorded outputs on retry.

**Architecture:** `ruse-storage` owns idempotency records. `ruse-core` owns an idempotent tool executor wrapper over existing `ToolLoop` behavior.

**Tech Stack:** Rust 2024 edition, `ruse-storage`, `ruse-core`, `ruse-tool`, `ruse-policy`, `serde_json`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Storage Tests

**Files:**
- Create: `crates/ruse-storage/tests/idempotency_store.rs`

- [x] **Step 1: Add failing store tests**

Tests must prove key validation, record/get behavior, duplicate same-key behavior, and mismatched input hash diagnostics.

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-storage idempotency_store
```

Expected: tests fail because idempotency contracts do not exist.

### Task 2: Implement Storage Contract

**Files:**
- Create: `crates/ruse-storage/src/idempotency.rs`
- Modify: `crates/ruse-storage/src/lib.rs`
- Modify: `crates/ruse-storage/src/memory.rs`
- Modify: `crates/ruse-storage/src/sqlite.rs`

- [x] **Step 1: Define `IdempotencyKey`, `ToolEffectRecord`, and `IdempotencyStore`**

Follow the fields in `docs/specs/M04-02-idempotency.spec.md`.

- [x] **Step 2: Implement in-memory and SQLite stores**

Store records by key and reject incompatible duplicate input hashes.

- [x] **Step 3: Verify storage GREEN**

```bash
cargo test -p ruse-storage idempotency_store
```

Expected: focused storage tests pass.

### Task 3: Add Core Idempotent Executor

**Files:**
- Create: `crates/ruse-core/src/idempotency.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Create: `crates/ruse-core/tests/idempotent_tool_replay.rs`

- [x] **Step 1: Add failing core tests**

Tests must prove first execution runs the tool, second execution with the same key returns stored output, and failed tool calls are not recorded.

- [x] **Step 2: Implement wrapper**

Wrap existing `ToolLoop` execution and persist `ToolEffectRecord` only after `ToolLoopStatus::Completed`.

- [x] **Step 3: Verify core GREEN**

```bash
cargo test -p ruse-core idempotent_tool_replay
```

Expected: focused core tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M04-02-idempotency.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M04-02-idempotency` to `M04-03-recovery-flow`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m04-02 idempotency"
```
