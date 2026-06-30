# M55-04 M56 Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create implementation-ready M56 planning so coding-agent golden-path work can start immediately after M55.

**Architecture:** Documentation planning slice. M56 must preserve shared runtime ownership and use CLI/TUI/Desktop as surfaces over the same core events and UI bridge contracts.

**Tech Stack:** Rust 2024 edition repository, Markdown docs, existing runtime/CLI/UI bridge architecture, `git diff --check`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M55-04-m56-bootstrap.spec.md`
- Read: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Review M56 bootstrap contract**

Run:

```bash
sed -n '1,260p' docs/specs/M55-04-m56-bootstrap.spec.md
```

Expected: M56 milestone and first-slice deliverables are clear.

### Task 2: Create M56 Milestone Planning

**Files:**
- Create: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Create: `docs/plans/M56-coding-agent-golden-path-v1.plan.md`

- [x] **Step 1: Write M56 milestone spec**

Define the vertical coding-agent journey, runtime ownership, UI surfaces, tool/policy/audit requirements, trace/replay/recovery expectations, and acceptance gates.

- [x] **Step 2: Write M56 milestone plan**

Decompose M56 only as needed. Expected starting slices:

- M56-01 repo session bootstrap
- M56-02 edit and diff flow
- M56-03 command/test loop
- M56-04 final review summary
- M56-05 golden path E2E

Adjust slice count only if M55 evidence proves a better breakdown.

### Task 3: Create First Implementation Slice

**Files:**
- Create: `docs/specs/M56-01-repo-session-bootstrap.spec.md`
- Create: `docs/plans/M56-01-repo-session-bootstrap.plan.md`

- [x] **Step 1: Write M56-01 spec**

Specify workspace input, durable session/conversation creation, Git/config/provider diagnostics, UI/JSON events, failure handling, security, and tests.

- [x] **Step 2: Write M56-01 plan**

Name concrete crates, tests, commands, and acceptance checklist so implementation can start without another planning pass.

### Task 4: Update Status and Commit

**Files:**
- Create: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Create: `docs/plans/M56-coding-agent-golden-path-v1.plan.md`
- Create: `docs/specs/M56-01-repo-session-bootstrap.spec.md`
- Create: `docs/plans/M56-01-repo-session-bootstrap.plan.md`
- Modify: `docs/plans/M55-04-m56-bootstrap.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Verify docs**

Run:

```bash
git diff --check -- docs/specs/M56-coding-agent-golden-path-v1.spec.md docs/plans/M56-coding-agent-golden-path-v1.plan.md docs/specs/M56-01-repo-session-bootstrap.spec.md docs/plans/M56-01-repo-session-bootstrap.plan.md docs/plans/M55-04-m56-bootstrap.plan.md docs/milestones/README.md
```

Expected: no whitespace errors.

- [ ] **Step 2: Commit**

Run exact-path staging only:

```bash
git add docs/specs/M56-coding-agent-golden-path-v1.spec.md docs/plans/M56-coding-agent-golden-path-v1.plan.md \
  docs/specs/M56-01-repo-session-bootstrap.spec.md docs/plans/M56-01-repo-session-bootstrap.plan.md \
  docs/plans/M55-04-m56-bootstrap.plan.md docs/milestones/README.md
git commit -m "docs: bootstrap m56 coding agent golden path"
```

Expected: commit contains only M55-04 and M56 planning docs/status updates.

### Completion Checklist

- [x] M56 milestone spec/plan exist.
- [x] M56-01 spec/plan are implementation-ready.
- [x] M56 scope maps to M55 baseline, smoke matrix, and backlog.
- [x] Milestone index points to M56 as the next implementation focus after M55.
- [x] `git diff --check` passes for changed docs.
