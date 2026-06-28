# M28 Evaluation Corpus and Quality Gates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the M27 single-scenario evaluation harness into scenario suites, built-in corpora, golden update workflows, CLI suite commands, and CI gates.

**Architecture:** `agent-eval` owns suite/golden/corpus logic. `agent-cli` and `xtask` are command surfaces over typed `agent-eval` contracts.

**Tech Stack:** Rust 2024 edition, serde, camino, existing M27 eval contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M28 Technical Design

**Files:**
- Create: `docs/specs/M28-00-evaluation-corpus-technical-design.spec.md`
- Create: `docs/plans/M28-00-evaluation-corpus-technical-design.plan.md`

- [x] **Step 1: Write the M28 technical design baseline**

Define suite ownership, corpus layout, golden update policy, CLI/xtask boundaries, privacy rules, and verification gates.

### Task 2: Decompose M28 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M28-01-scenario-suite-manifest.spec.md`
- Create: `docs/plans/M28-01-scenario-suite-manifest.plan.md`
- Create: `docs/specs/M28-02-builtin-eval-corpus.spec.md`
- Create: `docs/plans/M28-02-builtin-eval-corpus.plan.md`
- Create: `docs/specs/M28-03-golden-update-workflow.spec.md`
- Create: `docs/plans/M28-03-golden-update-workflow.plan.md`
- Create: `docs/specs/M28-04-cli-eval-suite-commands.spec.md`
- Create: `docs/plans/M28-04-cli-eval-suite-commands.plan.md`
- Create: `docs/specs/M28-05-xtask-eval-gate.spec.md`
- Create: `docs/plans/M28-05-xtask-eval-gate.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines contracts, failure handling, privacy behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan names files, focused tests, verification commands, and commit points.

### Task 3: Implement M28 Slices in Order

**Files:**
- Execute: `docs/plans/M28-01-scenario-suite-manifest.plan.md`
- Execute: `docs/plans/M28-02-builtin-eval-corpus.plan.md`
- Execute: `docs/plans/M28-03-golden-update-workflow.plan.md`
- Execute: `docs/plans/M28-04-cli-eval-suite-commands.plan.md`
- Execute: `docs/plans/M28-05-xtask-eval-gate.plan.md`

- [x] **Step 1: Implement scenario suite manifests**

Add versioned suite contracts, safe scenario path resolution, labels, strictness defaults, and serde tests.

- [ ] **Step 2: Add built-in eval corpus**

Check in a small suite and fixtures that exercise replay/evidence behavior.

- [ ] **Step 3: Add golden update workflow**

Make update mode explicit, typed, redacted, and separate from normal comparison.

- [ ] **Step 4: Add CLI eval suite commands**

Expose suite list, validate, run, and update-golden commands in text/JSON modes.

- [ ] **Step 5: Add `xtask` eval gate**

Provide a CI-friendly command plan that validates and runs the default corpus.

### Task 4: Complete M28

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M28-evaluation-corpus-quality-gates.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [ ] **Step 2: Mark M28 complete and commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md xtask
git commit -m "feat: complete m28 evaluation quality gates"
```
