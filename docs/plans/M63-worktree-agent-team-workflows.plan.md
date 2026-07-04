# M63 Worktree and Agent Team Workflows Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Productize safe parallel implementation and review through isolated worktrees, reduced-capability handoffs, shared blackboard output, reviewer decisions, conflict-aware merge readiness, CLI/JSON and UI-safe bridge surfaces, and deterministic smoke evidence.

**Architecture:** `agent-worktree` owns leases and conflict detection. `agent-team` owns team workflow, handoff, capability ceiling, blackboard, and review contracts. `agent-core` orchestrates snapshots and smoke fixtures. `agent-ui-bridge` exposes UI-safe frames. CLI/TUI/Desktop render typed summaries without duplicating policy, conflict, or readiness logic.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `agent-worktree`, `agent-team`, `agent-policy`, `agent-core`, `agent-ui-bridge`, CLI text/JSON commands, Ratatui and Desktop fixtures, Cargo fmt/clippy/test, Vitest, Vite build, and `cargo xtask eval`.

---

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M63-worktree-agent-team-workflows.spec.md`
- Read: `docs/PRD.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/specs/M09-worktree-subagents.spec.md`
- Read: `docs/specs/M62-reliability-recovery-productization.spec.md`
- Read: `crates/agent-worktree/`
- Read: `crates/agent-team/`
- Read: `crates/agent-core/`
- Read: `crates/agent-ui-bridge/`

- [x] **Step 1: Review existing foundations**

Confirm that M09 supplies worktree leases, conflict detection, handoff envelopes, capability reduction, and blackboard contracts, while M62 supplies recovery-safe conflict summaries and readiness behavior.

### Task 2: M63-01 Team Workflow Snapshot and Readiness Contracts

**Files:**
- Create: `docs/specs/M63-01-team-workflow-snapshot-readiness.spec.md`
- Create: `docs/plans/M63-01-team-workflow-snapshot-readiness.plan.md`
- Modify: `crates/agent-team/`
- Modify: `crates/agent-core/`
- Modify if needed: `crates/agent-ui-bridge/`

- [x] **Step 1: Plan M63-01**

Define the runtime/team snapshot contract that composes worktree leases, handoff envelopes, blackboard entries, review votes, conflict summaries, merge readiness, and capability ceilings.

- [x] **Step 2: Implement M63-01**

Add typed, serializable, bounded, redacted contracts and focused tests. Keep side effects out of M63-01; this slice should make the workflow state inspectable before CLI or UI commands are added.

Expected checks:

```bash
cargo test -p agent-team --test team_workflow_snapshot
cargo test -p agent-team
cargo clippy -p agent-team --all-targets -- -D warnings
```

Result: M63-01 is implemented in `agent-team` with workflow snapshot/readiness contracts, capability ceiling summaries, redaction/bounding helpers, and focused tests. Core assembly was not required in this slice.

### Task 3: M63-02 CLI and JSON Team Workflow Commands

**Files:**
- Create: `docs/specs/M63-02-cli-json-team-workflows.spec.md`
- Create: `docs/plans/M63-02-cli-json-team-workflows.plan.md`
- Modify: `crates/agent-cli/`
- Modify: `crates/agent-ui-bridge/`
- Modify: `docs/cli/`

- [x] **Step 1: Plan CLI/JSON surfaces**

Define commands for team status, handoff dry-run, blackboard inspect, reviewer vote record/dry-run, and merge-readiness dry-run.

- [x] **Step 2: Implement CLI/JSON surfaces**

Commands must render stable text and JSON frames from shared runtime/bridge snapshots.

Result: M63-02 adds deterministic `team` CLI commands, a `team_workflow_snapshot` bridge JSON frame, dry-run guards for handoff/review/merge-readiness commands, sample/conflict/policy-blocked fixtures, focused CLI/bridge tests, and minimal TUI frame consumers for exhaustive match safety.

### Task 4: M63-03 Reviewer Role and Merge Readiness

**Files:**
- Create: `docs/specs/M63-03-reviewer-merge-readiness.spec.md`
- Create: `docs/plans/M63-03-reviewer-merge-readiness.plan.md`
- Modify: `crates/agent-team/`
- Modify: `crates/agent-worktree/`
- Modify: `crates/agent-core/`

- [x] **Step 1: Plan reviewer and merge-readiness rules**

Define reviewer role, required artifact evidence, review votes, stale lease detection, conflict handling, and policy-backed readiness states.

- [x] **Step 2: Implement dry-run merge readiness**

Merge readiness must block on conflicts, missing review, pending approval, stale leases, policy denial, and incomplete handoffs.

Result: M63-03 adds `agent-team` merge readiness reports with typed blockers for policy, conflicts, pending approvals, stale worktrees, waiting/failed handoffs, blocking reviews, missing reviews, and missing artifacts. Workflow snapshots now include pending approval counts and block stale worktrees/pending approvals during readiness derivation.

### Task 5: M63-04 TUI/Desktop Team Surfaces

**Files:**
- Create: `docs/specs/M63-04-tui-desktop-team-surfaces.spec.md`
- Create: `docs/plans/M63-04-tui-desktop-team-surfaces.plan.md`
- Modify: `crates/agent-tui/`
- Modify: `crates/agent-desktop/`
- Modify: `apps/desktop/`
- Modify: `docs/desktop/`

- [x] **Step 1: Plan UI-safe team surfaces**

Define task/team panels that display team readiness, handoffs, subagent state, blackboard entries, review votes, conflicts, and merge-readiness next actions.

- [x] **Step 2: Implement TUI/Desktop rendering**

TUI and Desktop must render bridge snapshots only and must not implement readiness, policy, or conflict classification.

Result: M63-04 adds TUI task panel rendering for `team_workflow_snapshot` bridge frames, Desktop team workflow snapshot typing and state reduction, a compact Tasks-route team workflow panel, and focused TUI/Desktop tests. UI surfaces render shared bridge snapshots only; readiness and merge logic remain in shared crates.

### Task 6: M63-05 Team Workflow Smoke, Docs, and Closeout

**Files:**
- Create: `docs/specs/M63-05-team-workflow-smoke-closeout.spec.md`
- Create: `docs/plans/M63-05-team-workflow-smoke-closeout.plan.md`
- Modify: `docs/eval/`
- Modify: `docs/cli/`
- Modify: `docs/desktop/`
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Add deterministic smoke coverage**

Add credential-free evidence for isolated worktree planning, reduced-capability handoff, blackboard output, reviewer vote, conflict-aware merge readiness, and redaction.

- [x] **Step 2: Document user workflows**

Document CLI/Desktop team workflow usage, readiness states, review decisions, merge dry-runs, conflict handling, and redaction expectations.

- [x] **Step 3: Run final M63 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
```

Expected: M63 is productized, documented, smoke-tested, and ready to hand off to M64.

Result: M63-05 adds the `team_workflow_smoke` eval scenario and default-suite golden, CLI/Desktop team workflow docs, README and smoke matrix links, and final closeout status. Final gates passed with `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, and `cargo xtask eval`; the default eval suite reports `total=6 passed=6 failed=0 skipped=0 invalid=0 not_run=0`, including `default-m63-team-workflow-smoke`.

### Task 7: Commit Discipline

**Files:**
- Modify/create only files for the active slice and required status docs.

- [ ] **Step 1: Commit milestone planning**

Exact-stage only M63 milestone and M63-01 planning docs plus milestone index updates:

```bash
git status --short --branch
git diff --check
git add docs/specs/M63-worktree-agent-team-workflows.spec.md docs/plans/M63-worktree-agent-team-workflows.plan.md docs/specs/M63-01-team-workflow-snapshot-readiness.spec.md docs/plans/M63-01-team-workflow-snapshot-readiness.plan.md docs/milestones/README.md
git commit -m "docs: add m63 team workflow plan"
```

Expected: planning docs are committed before implementation slices begin.

- [ ] **Step 2: Commit each implementation slice**

For each M63 implementation slice, exact-stage the slice spec, slice plan, code, tests, docs, eval fixtures, and status files touched by that slice only.
