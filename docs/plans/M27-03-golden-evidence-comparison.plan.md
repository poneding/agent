# M27-03 Golden Evidence Comparison Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable golden comparison for evaluation reports and evidence summaries.

**Architecture:** `ruse-eval::golden` compares typed report/evidence data and returns typed mismatch diagnostics. CLI and CI render those diagnostics later.

**Tech Stack:** Rust 2024 edition, serde_json, redaction helpers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M27-03-golden-evidence-comparison.spec.md`
- Read: `crates/ruse-eval/src/report.rs`

- [x] **Step 1: Review report contracts**

Expected: comparison inputs are typed and serializable.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-eval/tests/golden.rs`

- [x] **Step 1: Add comparison tests**

Cover exact match, mismatch path, missing golden, malformed golden, and redaction.

### Task 3: Implement Comparison

**Files:**
- Create: `crates/ruse-eval/src/golden.rs`
- Modify: `crates/ruse-eval/src/lib.rs`

- [x] **Step 1: Add golden contracts**
- [x] **Step 2: Add selected-field comparison**
- [x] **Step 3: Add redacted mismatch diagnostics**

### Task 4: Verify and Commit

- [x] **Step 1: Run checks**

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-eval --all-targets -- -D warnings
cargo test -p ruse-eval golden
```

- [x] **Step 2: Commit**

```bash
git add Cargo.toml Cargo.lock crates/ruse-eval docs
git commit -m "feat: add eval golden comparison"
```
