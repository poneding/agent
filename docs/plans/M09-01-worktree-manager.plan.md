# M09-01 Worktree Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-worktree` with deterministic worktree lease lifecycle contracts.

**Architecture:** `ruse-worktree` depends on `ruse-types` only in this slice. Real Git side effects remain out of scope; the in-memory manager defines the durable API shape.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `thiserror`, existing `ruse-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-worktree`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-worktree/Cargo.toml`
- Create: `crates/ruse-worktree/src/lib.rs`
- Create: `crates/ruse-worktree/src/error.rs`

- [x] **Step 1: Add crate scaffold**

Add `crates/ruse-worktree` to workspace members.

Create the crate with dependencies:

```toml
[dependencies]
ruse-types = { path = "../ruse-types" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

### Task 2: Write Worktree Manager RED Tests

**Files:**
- Create: `crates/ruse-worktree/tests/worktree_manager.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `worktree_manager_serializes_lease_shape`
- `worktree_manager_plans_activates_and_releases_lease`
- `worktree_manager_rejects_invalid_request`
- `worktree_manager_rejects_unknown_lease`
- `worktree_manager_rejects_duplicate_release`

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-worktree worktree_manager
```

Expected: fails because `ruse-worktree` contracts do not exist.

### Task 3: Implement Worktree Contracts

**Files:**
- Create: `crates/ruse-worktree/src/lease.rs`
- Create: `crates/ruse-worktree/src/manager.rs`
- Modify: `crates/ruse-worktree/src/lib.rs`

- [x] **Step 1: Add lease contracts**

Implement `WorktreeLeaseId`, `WorktreeLeaseState`, `WorktreeCleanupPolicy`, `WorktreeRequest`, and `WorktreeLease`.

- [x] **Step 2: Add manager behavior**

Implement `WorktreeManager` and `InMemoryWorktreeManager`.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p ruse-worktree worktree_manager
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M09-01-worktree-manager.plan.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M09-02-conflict-detection`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m09 worktree manager"
```
