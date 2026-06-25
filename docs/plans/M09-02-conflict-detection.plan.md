# M09-02 Conflict Detection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure conflict detection for lead/subagent changed-file sets.

**Architecture:** Conflict contracts live in `agent-worktree` and build on the crate created by M09-01. No Git merge, filesystem scan, CLI, or UI behavior is included.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-worktree`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M09-02-conflict-detection.spec.md`
- Read: `crates/agent-worktree/src/lib.rs`

- [ ] **Step 1: Review the spec**

```bash
sed -n '1,260p' docs/specs/M09-02-conflict-detection.spec.md
```

### Task 2: Write Conflict RED Tests

**Files:**
- Create: `crates/agent-worktree/tests/conflict_detection.rs`

- [ ] **Step 1: Add failing tests**

Cover:

- `conflict_detection_reports_no_conflicts_for_distinct_paths`
- `conflict_detection_reports_both_modified`
- `conflict_detection_reports_delete_modify`
- `conflict_detection_allows_add_add_same_hash`
- `conflict_detection_reports_add_add_different_hash`
- `conflict_detection_rejects_invalid_paths`

- [ ] **Step 2: Verify RED**

```bash
cargo test -p agent-worktree conflict_detection
```

Expected: fails because conflict APIs do not exist.

### Task 3: Implement Conflict Detection

**Files:**
- Create: `crates/agent-worktree/src/conflict.rs`
- Modify: `crates/agent-worktree/src/lib.rs`

- [ ] **Step 1: Add conflict contracts**

Implement `FileChange`, `ChangeKind`, `ConflictKind`, `Conflict`, and `ConflictReport`.

- [ ] **Step 2: Add pure detector**

Implement `detect_conflicts` with stable path sorting and validation.

- [ ] **Step 3: Verify GREEN**

```bash
cargo test -p agent-worktree conflict_detection
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M09-02-conflict-detection.plan.md`

- [ ] **Step 1: Advance active focus**

Set active focus to `M09-03-subagent-handoff`.

- [ ] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m09 conflict detection"
```
