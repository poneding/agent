# M55 Product Baseline and Roadmap Reset Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a measurable mainline product baseline and prepare M56 implementation without continuing MCP-driven roadmap drift.

**Architecture:** M55 is docs-first and verification-first. Product baseline artifacts live in `docs/roadmap/`; milestone coordination lives in `docs/milestones/`; specs live in `docs/specs/`; plans live in `docs/plans/`. Runtime logic must remain in crates, and any executable smoke helper must use existing `xtask`, CLI, or eval entrypoints.

**Tech Stack:** Rust 2024 edition, Cargo workspace, Markdown docs, existing `xtask` and CLI verification commands, `git diff --check`, focused Cargo checks only if code changes are introduced.

---

### Task 1: Confirm M55 Inputs

**Files:**
- Read: `AGENTS.md`
- Read: `README.md`
- Read: `docs/PRD.md`
- Read: `docs/architecture/README.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/templates/spec-template.md`
- Read: `docs/templates/plan-template.md`

- [x] **Step 1: Review mainline direction**

Run:

```bash
sed -n '1,260p' docs/roadmap/PRODUCT_ROADMAP.md
sed -n '1,220p' docs/milestones/README.md
```

Expected: M55-M66 are the visible forward path and MCP expansion is parked after M54 except for product-blocking needs.

### Task 2: Establish M55 Milestone Planning Baseline

**Files:**
- Create: `docs/specs/M55-product-baseline-roadmap-reset.spec.md`
- Create: `docs/plans/M55-product-baseline-roadmap-reset.plan.md`

- [x] **Step 1: Write milestone-level spec and plan**

Define purpose, product context, scope, documentation contracts, state handling, security/privacy, testing strategy, and acceptance criteria.

- [x] **Step 2: Verify and commit the milestone planning baseline**

Run:

```bash
git diff --check -- docs/specs/M55-product-baseline-roadmap-reset.spec.md docs/plans/M55-product-baseline-roadmap-reset.plan.md
git add docs/specs/M55-product-baseline-roadmap-reset.spec.md docs/plans/M55-product-baseline-roadmap-reset.plan.md
git commit -m "docs: plan m55 product baseline reset"
```

Expected: commit includes only the M55 milestone-level spec/plan and does not stage MX01 MCP drafts.

### Task 3: Decompose M55 Into Necessary Slices

**Files:**
- Create: `docs/specs/M55-01-capability-inventory.spec.md`
- Create: `docs/plans/M55-01-capability-inventory.plan.md`
- Create: `docs/specs/M55-02-golden-path-smoke-matrix.spec.md`
- Create: `docs/plans/M55-02-golden-path-smoke-matrix.plan.md`
- Create: `docs/specs/M55-03-product-gap-backlog.spec.md`
- Create: `docs/plans/M55-03-product-gap-backlog.plan.md`
- Create: `docs/specs/M55-04-m56-bootstrap.spec.md`
- Create: `docs/plans/M55-04-m56-bootstrap.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Create slice specs**

Use four slices because M55 has four independent product outcomes: inventory, smoke matrix, backlog, and M56 bootstrap.

- [x] **Step 2: Create slice plans**

Each plan must name concrete files, implementation steps, verification commands, and completion checklist.

- [x] **Step 3: Update milestone index**

Add M55 to the milestone-level index and add an M55 slice index without marking implementation complete.

- [x] **Step 4: Verify and commit slice planning**

Run:

```bash
git diff --check -- docs/specs/M55-*.spec.md docs/plans/M55-*.plan.md docs/milestones/README.md
git add docs/specs/M55-01-capability-inventory.spec.md docs/plans/M55-01-capability-inventory.plan.md \
  docs/specs/M55-02-golden-path-smoke-matrix.spec.md docs/plans/M55-02-golden-path-smoke-matrix.plan.md \
  docs/specs/M55-03-product-gap-backlog.spec.md docs/plans/M55-03-product-gap-backlog.plan.md \
  docs/specs/M55-04-m56-bootstrap.spec.md docs/plans/M55-04-m56-bootstrap.plan.md \
  docs/milestones/README.md
git commit -m "docs: decompose m55 product baseline reset"
```

Expected: commit includes only M55 slice planning and milestone index updates.

### Task 4: Execute M55 Slices

**Files:**
- Execute: `docs/plans/M55-01-capability-inventory.plan.md`
- Execute: `docs/plans/M55-02-golden-path-smoke-matrix.plan.md`
- Execute: `docs/plans/M55-03-product-gap-backlog.plan.md`
- Execute: `docs/plans/M55-04-m56-bootstrap.plan.md`

- [x] **Step 1: Complete M55-01 capability inventory**

Produce `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md` from repository evidence.

- [x] **Step 2: Complete M55-02 smoke matrix**

Produce `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md` and add an executable helper only if an existing command cannot represent the smoke path.

- [x] **Step 3: Complete M55-03 product gap backlog**

Produce `docs/roadmap/PRODUCT_GAP_BACKLOG.md` with P0/P1/P2 ownership by M56-M66.

- [x] **Step 4: Complete M55-04 M56 bootstrap**

Create M56 milestone-level spec/plan and the minimal first implementation slice specs/plans needed to start coding.

### Task 5: Complete M55 Hardening

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M55-product-baseline-roadmap-reset.plan.md`

- [x] **Step 1: Reconcile status across docs**

Ensure README, PRD references, roadmap, milestone status, M55 artifacts, and M56 bootstrap agree on the next product direction.

- [x] **Step 2: Run final verification**

Run:

```bash
git diff --check
```

If code changed, also run the focused commands named by the changed slice and then:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: docs-only changes pass whitespace checks; code changes pass focused and workspace gates.

- [x] **Step 3: Commit M55 completion**

Run exact-path `git add` commands for M55 files only, then:

```bash
git commit -m "docs: complete m55 product baseline reset"
```

Expected: M55 completion commit excludes MX01 MCP epilogue drafts unless the user explicitly moves that work into scope.
