# M29 CI Quality Evidence and Release Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a shared local/CI quality gate that runs checks, evals, and writes durable evidence artifacts.

**Architecture:** `xtask` owns gate orchestration and artifacts. Runtime crates keep their existing boundaries; CI invokes `xtask` rather than duplicating command logic.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde JSON, GitHub Actions YAML, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M29 Technical Design

**Files:**
- Create: `docs/specs/M29-00-ci-quality-technical-design.spec.md`
- Create: `docs/plans/M29-00-ci-quality-technical-design.plan.md`

- [x] **Step 1: Write the M29 technical design baseline**

Define gate ownership, report contracts, command execution rules, privacy, CI workflow boundaries, and verification gates.

### Task 2: Decompose M29 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M29-01-quality-gate-report-contracts.spec.md`
- Create: `docs/plans/M29-01-quality-gate-report-contracts.plan.md`
- Create: `docs/specs/M29-02-xtask-ci-runner.spec.md`
- Create: `docs/plans/M29-02-xtask-ci-runner.plan.md`
- Create: `docs/specs/M29-03-github-actions-workflow.spec.md`
- Create: `docs/plans/M29-03-github-actions-workflow.plan.md`
- Create: `docs/specs/M29-04-quality-artifact-retention.spec.md`
- Create: `docs/plans/M29-04-quality-artifact-retention.plan.md`
- Create: `docs/specs/M29-05-release-readiness-docs.spec.md`
- Create: `docs/plans/M29-05-release-readiness-docs.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines contracts, failure handling, privacy behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan names files, focused tests, verification commands, and commit points.

### Task 3: Implement M29 Slices in Order

**Files:**
- Execute: `docs/plans/M29-01-quality-gate-report-contracts.plan.md`
- Execute: `docs/plans/M29-02-xtask-ci-runner.plan.md`
- Execute: `docs/plans/M29-03-github-actions-workflow.plan.md`
- Execute: `docs/plans/M29-04-quality-artifact-retention.plan.md`
- Execute: `docs/plans/M29-05-release-readiness-docs.plan.md`

- [x] **Step 1: Add quality gate report contracts**

Add typed report structs, status aggregation, and JSON/text rendering tests.

- [x] **Step 2: Add `xtask ci` runner**

Run fmt, clippy, tests, and eval through a single evidence-producing command.

- [x] **Step 3: Add hosted CI workflow**

Add workflow configuration that invokes the shared `xtask ci` gate.

- [x] **Step 4: Add artifact retention behavior**

Persist JSON and text summaries under `target/agent-ci` and document upload paths.

- [x] **Step 5: Complete release-readiness docs**

Update README, milestones, and final gate docs.

### Task 4: Complete M29

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M29-ci-quality-evidence-release-gates.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
cargo xtask eval
```

Expected: all checks pass.

- [x] **Step 2: Mark M29 complete and commit**

Run:

```bash
git add .github README.md docs xtask
git commit -m "feat: complete m29 ci quality gates"
```
