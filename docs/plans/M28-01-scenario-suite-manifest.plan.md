# M28-01 Scenario Suite Manifest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add versioned scenario suite manifests to `ruse-eval`.

**Architecture:** `ruse-eval::suite` validates suite manifests and calls `ruse-eval::manifest` for referenced scenario files. No CLI logic lives in the suite module.

**Tech Stack:** Rust 2024 edition, serde, camino, existing M27 manifest contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M28-01-scenario-suite-manifest.spec.md`
- Read: `crates/ruse-eval/src/manifest.rs`

- [x] **Step 1: Review contracts**

Expected: suite code can reuse existing scenario validation.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-eval/tests/suite.rs`

- [x] **Step 1: Add suite serde and validation tests**

Cover valid expansion, unsupported schema, invalid ID, duplicate aliases, missing files, path traversal, labels, and default strictness.

### Task 3: Implement Suite Contracts

**Files:**
- Create: `crates/ruse-eval/src/suite.rs`
- Modify: `crates/ruse-eval/src/lib.rs`

- [x] **Step 1: Add public suite structs and errors**
- [x] **Step 2: Add suite validation and safe path normalization**
- [x] **Step 3: Add scenario expansion using existing manifest validation**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused checks**

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-eval --all-targets -- -D warnings
cargo test -p ruse-eval --test suite
```

- [x] **Step 2: Commit**

```bash
git add crates/ruse-eval docs
git commit -m "feat: add eval scenario suites"
```
