# M27-05 CI Reporting Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate CI-friendly evaluation artifacts and complete M27.

**Architecture:** `agent-eval` writes report artifacts. CLI or xtask invokes the same typed writer. Documentation explains the workflow.

**Tech Stack:** Rust 2024 edition, serde_json, atomic file writes where practical, Cargo fmt, Cargo clippy, Cargo test, cargo xtask check.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-05-ci-reporting-integration.spec.md`
- Read: `crates/agent-eval/src/report.rs`
- Read: `xtask/src/main.rs`

- [x] **Step 1: Review report and xtask patterns**

Expected: artifact generation can reuse existing command infrastructure.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-eval/tests/report.rs`

- [x] **Step 1: Add artifact writer tests**

Cover JSON artifact, summary artifact, empty run, failure counts, and unsafe output path.

### Task 3: Implement Artifact Writer

**Files:**
- Modify: `crates/agent-eval/src/report.rs`
- Modify: `crates/agent-eval/src/lib.rs`
- Modify as needed: `xtask/src/main.rs`

- [x] **Step 1: Add artifact writer contracts**
- [x] **Step 2: Add JSON and summary writers**
- [x] **Step 3: Wire a CI-friendly command if aligned with existing xtask style**

### Task 4: Document and Complete M27

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M27-runtime-evaluation-harness.plan.md`

- [x] **Step 1: Document eval workflow**
- [x] **Step 2: Mark M27 complete after final checks**

### Task 5: Verify and Commit

- [x] **Step 1: Run final workspace checks**

```bash
cargo xtask check
```

Expected: all checks pass.

- [x] **Step 2: Commit M27 completion**

```bash
git add Cargo.toml Cargo.lock crates xtask docs README.md
git commit -m "feat: complete m27 evaluation harness"
```
