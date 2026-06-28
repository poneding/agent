# M24 Background Agents UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make long-running and scheduled agent tasks manageable from CLI, TUI, and Desktop-facing bridge surfaces.

**Architecture:** `agent-scheduler` owns task state, queues, leases, retries, and quotas. `agent-core` runs task turns. `agent-ui-bridge` exposes task commands/events. `agent-worktree` provides optional coding-task isolation.

**Tech Stack:** Rust 2024 edition, durable scheduler contracts, Tokio task runners, serde UI schemas, event storage, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M24 Technical Design

**Files:**
- Create: `docs/specs/M24-00-background-agents-technical-design.spec.md`
- Create: `docs/plans/M24-00-background-agents-technical-design.plan.md`

- [x] **Step 1: Write the M24 technical design baseline**

Define task runner integration, approval waiting, notifications, quotas, artifacts, schedules, retries, and recovery.

- [x] **Step 2: Write the M24 design execution plan**

Document slice order and recovery test strategy.

### Task 2: Decompose M24 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M24-01-task-command-bridge.spec.md`
- Create: `docs/plans/M24-01-task-command-bridge.plan.md`
- Create: `docs/specs/M24-02-background-turn-runner.spec.md`
- Create: `docs/plans/M24-02-background-turn-runner.plan.md`
- Create: `docs/specs/M24-03-task-approvals-notifications.spec.md`
- Create: `docs/plans/M24-03-task-approvals-notifications.plan.md`
- Create: `docs/specs/M24-04-task-quotas-retries-recovery.spec.md`
- Create: `docs/plans/M24-04-task-quotas-retries-recovery.plan.md`
- Create: `docs/specs/M24-05-task-artifacts-review.spec.md`
- Create: `docs/plans/M24-05-task-artifacts-review.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines task state, commands, events, failure behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan includes scheduler integration tests and recovery commands.

### Task 3: Implement M24 Slices in Order

**Files:**
- Execute: `docs/plans/M24-01-task-command-bridge.plan.md`
- Execute: `docs/plans/M24-02-background-turn-runner.plan.md`
- Execute: `docs/plans/M24-03-task-approvals-notifications.plan.md`
- Execute: `docs/plans/M24-04-task-quotas-retries-recovery.plan.md`
- Execute: `docs/plans/M24-05-task-artifacts-review.plan.md`

- [x] **Step 1: Add task command bridge**

Expose create, list, pause, resume, cancel, inspect, and schedule commands.

- [x] **Step 2: Add background turn runner**

Connect scheduler leases to comprehensive turn execution.

- [x] **Step 3: Add approvals and notifications**

Pause tasks for approvals and notify UI clients.

- [x] **Step 4: Add quotas, retries, and recovery**

Enforce budgets and recover from stale leases.

- [x] **Step 5: Add artifacts and review summaries**

Persist task outputs, traces, and completion summaries.

### Task 4: Complete M24

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M24-background-agents-ux.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Mark M24 complete**

Update milestone status and advance active focus to M25.

- [x] **Step 3: Commit M24 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m24 background agents"
```

Expected: commit succeeds.
