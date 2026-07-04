# M10-03 Permissions and Diffs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Desktop permission dialog and diff review contracts with explicit state transitions.

**Architecture:** Implement permission and diff contracts inside `agent-desktop`. Add an `agent-policy` dependency only if it is not already present, and keep actual approval execution in existing runtime/UI bridge paths.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, existing `agent-desktop`, `agent-types`, `agent-policy`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M10-03-permissions-diffs.spec.md`
- Read: `crates/agent-policy/src/request.rs`
- Read: `crates/agent-tool/src/builtins/edit_file.rs`

- [x] **Step 1: Review the spec**

```bash
sed -n '1,280p' docs/specs/M10-03-permissions-diffs.spec.md
```

### Task 2: Write Permissions and Diffs RED Tests

**Files:**
- Create: `crates/agent-desktop/tests/permissions_diffs.rs`

- [x] **Step 1: Add failing tests**

Cover:

- `permissions_diffs_serializes_permission_dialog_shape`
- `permissions_diffs_transitions_permission_decisions`
- `permissions_diffs_rejects_terminal_permission_transition`
- `permissions_diffs_serializes_diff_review_shape`
- `permissions_diffs_rejects_unsafe_diff_paths`

- [x] **Step 2: Verify RED**

```bash
cargo test -p agent-desktop permissions_diffs
```

Expected: fails because permission and diff contracts do not exist.

### Task 3: Implement Permission and Diff Contracts

**Files:**
- Modify: `crates/agent-desktop/Cargo.toml`
- Create: `crates/agent-desktop/src/permission.rs`
- Create: `crates/agent-desktop/src/diff.rs`
- Modify: `crates/agent-desktop/src/lib.rs`
- Modify: `crates/agent-desktop/src/error.rs`

- [x] **Step 1: Add permission contracts**

Implement `PermissionDialogStatus`, `DesktopPermissionDecision`, `PermissionDialog`, and permission transition helpers.

- [x] **Step 2: Add diff review contracts**

Implement `DiffReviewStatus`, `DiffFileSummary`, `DiffReview`, and diff transition helpers.

- [x] **Step 3: Verify GREEN**

```bash
cargo test -p agent-desktop permissions_diffs
```

### Task 4: Update Progress and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M10-03-permissions-diffs.plan.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M10-04-trace-timeline`.

- [x] **Step 2: Run standard verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m10 permissions diffs"
```
