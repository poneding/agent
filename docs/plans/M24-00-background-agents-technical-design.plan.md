# M24-00 Background Agents Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M24 architecture baseline for background and scheduled agent UX.

**Architecture:** This is a documentation-only slice. It defines scheduler/core ownership, task commands, background runner, approvals, quotas, retries, recovery, artifacts, and UI bridge task events.

**Tech Stack:** Markdown docs, existing scheduler/core/worktree architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-background-agents-ux.spec.md`
- Read: `docs/specs/M08-scheduler-background-tasks.spec.md`
- Read: `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md`
- Read: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`

- [x] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M24-background-agents-ux.spec.md
```

Expected: milestone scope defines task commands, background turns, approvals, quotas, retries, and artifacts.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M24-00-background-agents-technical-design.spec.md`

- [x] **Step 1: Define ownership and contracts**

Document task model, command surface, runner behavior, approvals, quotas, recovery, security, and testing strategy.

- [x] **Step 2: Self-review design**

Check that recurring schedules cannot broaden permissions and stale leases recover deterministically.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M24-00-background-agents-technical-design.spec.md`

- [x] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M24-00-background-agents-technical-design.spec.md`
- Add: `docs/plans/M24-00-background-agents-technical-design.plan.md`
- Modify: `docs/plans/M24-background-agents-ux.plan.md`

- [x] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M24-00-background-agents-technical-design.spec.md docs/plans/M24-00-background-agents-technical-design.plan.md docs/plans/M24-background-agents-ux.plan.md
git commit -m "docs: add m24 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
