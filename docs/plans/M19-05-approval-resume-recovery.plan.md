# M19-05 Approval Resume and Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist pending approvals, suspend turns, and resume side-effecting tool execution after approval or denial.

**Architecture:** `agent-core` owns suspended turn orchestration and event emission. `agent-policy` validates approval state transitions and lease scope. `agent-storage` persists event/checkpoint evidence through existing stores.

**Tech Stack:** Rust 2024 edition, Tokio tests, existing comprehensive turn runner, checkpoint store, idempotency store, serde runtime events, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-05-approval-resume-recovery.spec.md`
- Read: `crates/agent-core/src/model_runtime.rs`
- Read: `crates/agent-core/src/tool_loop.rs`
- Read: `crates/agent-storage/src/checkpoint.rs`

- [x] **Step 1: Review resume scope**

Run:

```bash
sed -n '1,240p' docs/specs/M19-05-approval-resume-recovery.spec.md
rg -n "WaitingApproval|run_with_idempotency|Checkpoint" crates/agent-core/src crates/agent-storage/src
```

Expected: this slice wires approval contracts into core suspension and resume.

### Task 2: Write RED Core Tests

**Files:**
- Create: `crates/agent-core/tests/approval_resume_recovery.rs`

- [x] **Step 1: Add approval resume tests**

Add tests for ask -> pending -> approve -> execute, ask -> deny -> no execution, expired response rejection, duplicate approval rejection, and idempotent replay after resume.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core --test approval_resume_recovery
```

Expected: compile or test failures because core approval resume is not implemented.

### Task 3: Add Runtime Events and Pending Store Boundary

**Files:**
- Modify: `crates/agent-types/src/event.rs`
- Create: `crates/agent-core/src/approval.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add approval runtime events**

Add approval requested, resolved, expired, cancelled, and consumed payloads with redacted summaries.

- [x] **Step 2: Add in-core pending approval coordinator**

Create a coordinator that maps `PolicyDecision::Ask` to `PendingApproval` plus checkpoint reference.

### Task 4: Wire Resume Execution

**Files:**
- Modify: `crates/agent-core/src/tool_loop.rs`
- Modify: `crates/agent-core/src/model_runtime.rs`

- [x] **Step 1: Suspend on ask with pending approval**

Return a terminal waiting state containing pending approval metadata instead of losing the approval context.

- [x] **Step 2: Resume after approval**

Re-load pending approval, create scoped lease, re-check idempotency, and execute through the normal tool path.

- [x] **Step 3: Resume after denial**

Record denial and provide a bounded model-visible observation when the loop can continue.

### Task 5: Verify

**Files:**
- Read: `crates/agent-core/tests/approval_resume_recovery.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core --test approval_resume_recovery
```

Expected: focused tests pass.

- [x] **Step 2: Run M19 core regression tests**

Run:

```bash
cargo test -p agent-core --test comprehensive_turn_observations
cargo test -p agent-core --test comprehensive_turn_idempotency
cargo test -p agent-core --test comprehensive_turn_guards
```

Expected: existing comprehensive turn behavior remains intact.

### Task 6: Complete M19

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`

- [x] **Step 1: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: all workspace checks pass.

- [x] **Step 2: Mark M19 complete**

Update milestone status and set next focus to M20 Terminal UX and TUI REPL.
