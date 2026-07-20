# M55-02 Golden Path Smoke Matrix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a golden-path smoke matrix that makes product behavior verifiable across CLI/TUI/Desktop bridge and runtime paths.

**Architecture:** Documentation-first verification contract under `docs/roadmap/`. Prefer existing `cargo xtask`, CLI, eval, and crate test commands. Add no runtime logic in this slice unless a tiny existing command reference must be documented.

**Tech Stack:** Rust 2024 edition repository, Markdown docs, existing `xtask`, CLI, eval fixtures, `git diff --check`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M55-02-golden-path-smoke-matrix.spec.md`
- Read: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Read: `README.md`
- Read: `xtask/`
- Read: relevant CLI, TUI, Desktop bridge, eval, and runtime tests

- [x] **Step 1: Review required smoke groups**

Run:

```bash
sed -n '1,260p' docs/specs/M55-02-golden-path-smoke-matrix.spec.md
```

Expected: smoke row contract and status labels are clear.

### Task 2: Discover Existing Smoke Evidence

**Files:**
- Read: `README.md`
- Read: `xtask/`
- Read: `crates/ruse-cli/`
- Read: `crates/ruse-tui/`
- Read: `crates/ruse-ui-bridge/`
- Read: `crates/ruse-core/`
- Read: `crates/ruse-eval/` if present

- [x] **Step 1: Map available commands**

Run:

```bash
rg "cargo xtask|cli-json|cli-prompt|desktop-test|eval|replay|checkpoint|json" README.md xtask crates docs
rg "fake|fixture|golden|smoke|e2e|approval|permission|audit" crates docs xtask
```

Expected: a command or blocker can be named for each required smoke group.

### Task 3: Write Smoke Matrix

**Files:**
- Create: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [x] **Step 1: Create smoke matrix document**

Include:

- status legend
- smoke matrix table
- command notes
- required fixture/config notes
- immediate M56-M60 implications

- [x] **Step 2: Avoid overclaiming**

Mark rows as `Documented`, `Blocked`, or `Missing` when no fresh execution evidence exists.

### Task 4: Verify and Commit

**Files:**
- Create: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Modify: `docs/plans/M55-02-golden-path-smoke-matrix.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run docs verification**

Run:

```bash
git diff --check -- docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md docs/plans/M55-02-golden-path-smoke-matrix.plan.md docs/milestones/README.md
```

Expected: no whitespace errors.

- [x] **Step 2: Commit**

Run exact-path staging only:

```bash
git add docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md docs/plans/M55-02-golden-path-smoke-matrix.plan.md docs/milestones/README.md
git commit -m "docs: add m55 golden path smoke matrix"
```

Expected: commit contains only M55-02 docs and status updates.

### Completion Checklist

- [x] Required smoke groups are present.
- [x] Every row has a command path or blocker.
- [x] Real-provider checks are opt-in.
- [x] Product safety checks cover permission, audit, trace, replay, and recovery.
- [x] `git diff --check` passes for changed docs.
