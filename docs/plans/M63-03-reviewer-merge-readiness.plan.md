# M63-03 Reviewer Role and Merge Readiness Plan

> **For agentic workers:** Keep this slice contract-first. Do not implement live merge execution or UI panels.

**Goal:** Add typed reviewer/merge readiness reports with explicit blockers for review, artifact, conflict, stale worktree, pending approval, and policy states.

**Architecture:** `ruse-team` owns reviewer/merge-readiness contracts and readiness derivation. `ruse-worktree` remains the source for worktree lease/conflict primitives. CLI/TUI/Desktop continue to consume typed snapshots/frames without duplicating readiness logic.

---

### Task 1: Plan Contracts

**Files:**
- Create: `docs/specs/M63-03-reviewer-merge-readiness.spec.md`
- Create: `docs/plans/M63-03-reviewer-merge-readiness.plan.md`

- [x] **Step 1: Define blockers**

Define typed blockers for policy, conflicts, pending approvals, stale worktrees, waiting/failed handoffs, blocking reviews, missing reviews, and missing artifacts.

- [x] **Step 2: Define readiness report**

Define a merge readiness report derived from `TeamWorkflowSnapshot`.

### Task 2: Implement Agent-Team Contracts

**Files:**
- Modify: `crates/ruse-team/src/workflow.rs`
- Create: `crates/ruse-team/src/merge_review.rs`
- Modify: `crates/ruse-team/src/lib.rs`
- Create: `crates/ruse-team/tests/team_merge_readiness.rs`

- [x] **Step 1: Add pending approval support**

Add `pending_approval_count` to workflow snapshots and make pending approvals block readiness.

- [x] **Step 2: Add stale worktree support**

Make stale worktrees block merge readiness.

- [x] **Step 3: Add merge readiness report**

Add blocker kinds, blocker summaries, and report construction from snapshots.

- [x] **Step 4: Add deterministic tests**

Cover merge-ready, conflict, pending approval, stale worktree, blocking review, missing review, missing artifact, and redaction.

### Task 3: Verify

**Files:**
- Read: changed files

- [x] **Step 1: Format**

```bash
cargo fmt --all -- --check
```

- [x] **Step 2: Focused tests**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo test -p ruse-team --test team_merge_readiness
CARGO_TARGET_DIR=target/m63-gate cargo test -p ruse-team
```

- [x] **Step 3: Focused clippy**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo clippy -p ruse-team --all-targets -- -D warnings
```

- [x] **Step 4: Diff check**

```bash
git diff --check
```

### Task 4: Status And Commit

**Files:**
- Modify: `docs/plans/M63-worktree-ruse-team-workflows.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Update status docs after focused gates pass**

Mark M63-03 implementation complete only after focused gates pass. Keep M63 overall in progress.

Result: focused gates passed with `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=target/m63-gate cargo test -p ruse-team --test team_merge_readiness`, `CARGO_TARGET_DIR=target/m63-gate cargo test -p ruse-team`, `CARGO_TARGET_DIR=target/m63-gate cargo clippy -p ruse-team --all-targets -- -D warnings`, and `git diff --check`.

- [ ] **Step 2: Commit**

Exact-stage only M63-03 files. Exclude `tmp/`.

Suggested commit message:

```text
feat: add m63 merge readiness blockers
```
