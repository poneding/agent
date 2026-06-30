# M55-03 Product Gap Backlog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert M55 baseline and smoke findings into a prioritized backlog mapped to M56-M66.

**Architecture:** Documentation-only slice. Use M55-01 and M55-02 outputs as inputs; write backlog under `docs/roadmap/` and update milestone status when complete.

**Tech Stack:** Rust 2024 edition repository, Markdown docs, `git diff --check`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M55-03-product-gap-backlog.spec.md`
- Read: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`

- [ ] **Step 1: Review backlog contract**

Run:

```bash
sed -n '1,260p' docs/specs/M55-03-product-gap-backlog.spec.md
```

Expected: backlog item shape, priorities, and status rules are clear.

### Task 2: Extract Gaps

**Files:**
- Read: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [ ] **Step 1: Map evidence to gaps**

For each capability domain and smoke row, identify:

- missing product behavior
- missing verification evidence
- target milestone
- acceptance signal
- dependency

Expected: all P0 coding-agent daily-driver blockers are captured.

### Task 3: Write Product Backlog

**Files:**
- Create: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`

- [ ] **Step 1: Create backlog document**

Include:

- priority legend
- M56-M66 backlog tables
- deferred MCP epilogue notes
- cross-milestone dependencies
- M56 immediate start list

- [ ] **Step 2: Keep items product-shaped**

Write each item as a user-visible outcome with acceptance evidence, not as an internal implementation wish list.

### Task 4: Verify and Commit

**Files:**
- Create: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Modify: `docs/plans/M55-03-product-gap-backlog.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Run docs verification**

Run:

```bash
git diff --check -- docs/roadmap/PRODUCT_GAP_BACKLOG.md docs/plans/M55-03-product-gap-backlog.plan.md docs/milestones/README.md
```

Expected: no whitespace errors.

- [ ] **Step 2: Commit**

Run exact-path staging only:

```bash
git add docs/roadmap/PRODUCT_GAP_BACKLOG.md docs/plans/M55-03-product-gap-backlog.plan.md docs/milestones/README.md
git commit -m "docs: add m55 product gap backlog"
```

Expected: commit contains only M55-03 docs and status updates.

### Completion Checklist

- [ ] Every P0 has a target milestone and acceptance signal.
- [ ] M56 has a clear immediate start list.
- [ ] Later milestones have scoped backlog without premature implementation detail.
- [ ] MCP epilogue work is parked or explicitly product-linked.
- [ ] `git diff --check` passes for changed docs.
