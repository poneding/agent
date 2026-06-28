# M27 Runtime Evaluation Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a deterministic evaluation and regression harness over existing runtime replay, fake-model, local-coding, and observability evidence fixtures.

**Architecture:** `agent-eval` owns manifest, runner, golden comparison, and report contracts. `agent-cli` calls `agent-eval` for command surfaces. Runtime logic stays out of UI crates.

**Tech Stack:** Rust 2024 edition, serde, typed runtime events, replay fixtures, observability evidence, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M27 Technical Design

**Files:**
- Create: `docs/specs/M27-00-runtime-evaluation-technical-design.spec.md`
- Create: `docs/plans/M27-00-runtime-evaluation-technical-design.plan.md`

- [x] **Step 1: Write the M27 technical design baseline**

Define evaluation scenario kinds, fixture boundaries, deterministic execution rules, golden comparison policy, report contracts, CLI/CI surfaces, privacy rules, and verification gates.

### Task 2: Decompose M27 Into Slice Specs and Plans

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

- [x] **Step 1: Create slice specs**

Each slice defines concrete contracts, failure handling, privacy behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan names files, focused tests, verification commands, and commit points.

### Task 3: Implement M27 Slices in Order

**Files:**
- Execute: `docs/plans/M27-01-scenario-manifest-contracts.plan.md`
- Execute: `docs/plans/M27-02-scenario-runner-core.plan.md`
- Execute: `docs/plans/M27-03-golden-evidence-comparison.plan.md`
- Execute: `docs/plans/M27-04-cli-eval-commands.plan.md`
- Execute: `docs/plans/M27-05-ci-reporting-integration.plan.md`

- [ ] **Step 1: Implement scenario manifest contracts**

Add `agent-eval` crate, versioned scenario manifests, validation, path normalization, and serde tests.

- [ ] **Step 2: Implement scenario runner core**

Run deterministic replay, fake-model, local-coding, and evidence-only scenarios into typed reports.

- [ ] **Step 3: Implement golden evidence comparison**

Compare selected evidence/report fields against golden files with redacted, stable mismatch diagnostics.

- [ ] **Step 4: Implement CLI eval commands**

Expose list, validate, run, and report commands with text/JSON output and CI-safe exit codes.

- [ ] **Step 5: Implement CI/reporting integration**

Write machine-readable reports, summary tables, and check-friendly artifacts.

### Task 4: Complete M27

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M27-runtime-evaluation-harness.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [ ] **Step 2: Mark M27 complete**

Update milestone status and record the next implementation focus.

- [ ] **Step 3: Commit M27 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m27 evaluation harness"
```

Expected: commit succeeds.
