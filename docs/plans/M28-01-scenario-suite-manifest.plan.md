# M28-01 Scenario Suite Manifest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned scenario suite manifests to `agent-eval`.

**Architecture:** `agent-eval::suite` validates suite manifests and calls `agent-eval::manifest` for referenced scenario files. No CLI logic lives in the suite module.

**Tech Stack:** Rust 2024 edition, serde, camino, existing M27 manifest contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M28-01-scenario-suite-manifest.spec.md`
- Read: `crates/agent-eval/src/manifest.rs`

- [ ] **Step 1: Review contracts**

Expected: suite code can reuse existing scenario validation.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-eval/tests/suite.rs`

- [ ] **Step 1: Add suite serde and validation tests**

Cover valid expansion, unsupported schema, invalid ID, duplicate aliases, missing files, path traversal, labels, and default strictness.

### Task 3: Implement Suite Contracts

**Files:**
- Create: `crates/agent-eval/src/suite.rs`
- Modify: `crates/agent-eval/src/lib.rs`

- [ ] **Step 1: Add public suite structs and errors**
- [ ] **Step 2: Add suite validation and safe path normalization**
- [ ] **Step 3: Add scenario expansion using existing manifest validation**

### Task 4: Verify and Commit

- [ ] **Step 1: Run focused checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-eval --all-targets -- -D warnings
cargo test -p agent-eval --test suite
```

- [ ] **Step 2: Commit**

```bash
git add crates/agent-eval docs
git commit -m "feat: add eval scenario suites"
```
