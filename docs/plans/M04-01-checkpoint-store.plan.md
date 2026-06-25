# M04-01 Checkpoint Store Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable checkpoint records and checkpoint stores to `agent-storage`.

**Architecture:** `agent-storage` owns the checkpoint contract and implementations. `agent-core` will consume the trait in later slices; this slice does not add recovery orchestration.

**Tech Stack:** Rust 2024 edition, `agent-storage`, `agent-types`, `serde_json`, `rusqlite`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Checkpoint Store Tests

**Files:**
- Create: `crates/agent-storage/tests/checkpoint_store.rs`

- [ ] **Step 1: Add failing behavior tests**

Tests must prove serialization shape, in-memory put/latest, SQLite put/latest, duplicate rejection, and turn filtering.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-storage checkpoint_store
```

Expected: tests fail because `CheckpointRecord` and `CheckpointStore` do not exist.

### Task 2: Implement Checkpoint Contract

**Files:**
- Create: `crates/agent-storage/src/checkpoint.rs`
- Modify: `crates/agent-storage/src/lib.rs`

- [ ] **Step 1: Define `CheckpointRecord`**

Add the fields from `docs/specs/M04-01-checkpoint-store.spec.md` and derive `Clone`, `Debug`, `Eq`, `PartialEq`, `Serialize`, and `Deserialize`.

- [ ] **Step 2: Define `CheckpointStore`**

Add `put_checkpoint` and `latest_checkpoint` methods returning `StorageError`.

### Task 3: Implement Stores

**Files:**
- Modify: `crates/agent-storage/src/memory.rs`
- Modify: `crates/agent-storage/src/sqlite.rs`

- [ ] **Step 1: Add in-memory checkpoint storage**

Store checkpoints by ID and index them by `(conversation_id, turn_id)`.

- [ ] **Step 2: Add SQLite checkpoint table**

Create a `checkpoints` table with `checkpoint_id`, `conversation_id`, `turn_id`, `event_seq`, `label`, `state_json`, `schema_version`, and `created_at`.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-storage checkpoint_store
```

Expected: focused checkpoint tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M04-01-checkpoint-store.plan.md`

- [ ] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 2: Advance active focus**

Update active focus from `M04-01-checkpoint-store` to `M04-02-idempotency`.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m04-01 checkpoint store"
```
