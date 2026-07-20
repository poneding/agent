# M27-00 Runtime Evaluation Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the architecture baseline for M27 evaluation harness work.

**Architecture:** `ruse-eval` owns evaluation contracts and depends on runtime crates. `ruse-cli` and CI surfaces consume typed reports.

**Tech Stack:** Markdown specs/plans, Rust workspace architecture docs, Cargo workspace conventions.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-runtime-evaluation-harness.spec.md`
- Read: `docs/architecture/README.md`

- [x] **Step 1: Review architecture boundaries**

Expected: evaluation runtime logic belongs in a crate, not CLI/Desktop presentation layers.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M27-00-runtime-evaluation-technical-design.spec.md`

- [x] **Step 1: Define the owner crate and boundaries**

Expected: spec names scenario taxonomy, deterministic execution rules, report strategy, and security posture.

### Task 3: Decompose Slices

**Files:**
- Create: `docs/specs/M27-01-scenario-manifest-contracts.spec.md`
- Create: `docs/plans/M27-01-scenario-manifest-contracts.plan.md`
- Create: `docs/specs/M27-02-scenario-runner-core.spec.md`
- Create: `docs/plans/M27-02-scenario-runner-core.plan.md`
- Create: `docs/specs/M27-03-golden-evidence-comparison.spec.md`
- Create: `docs/plans/M27-03-golden-evidence-comparison.plan.md`
- Create: `docs/specs/M27-04-cli-eval-commands.spec.md`
- Create: `docs/plans/M27-04-cli-eval-commands.plan.md`
- Create: `docs/specs/M27-05-ci-reporting-integration.spec.md`
- Create: `docs/plans/M27-05-ci-reporting-integration.plan.md`

- [x] **Step 1: Create implementation slice specs and plans**

Expected: every slice can be implemented and committed independently.

### Task 4: Update Milestone Index

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M27 as active**

Expected: milestone status, index, slice index, and active work all reference M27.
