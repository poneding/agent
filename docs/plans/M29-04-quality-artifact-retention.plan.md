# M29-04 Quality Artifact Retention Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stabilize quality gate artifact paths and summaries.

**Architecture:** `xtask` writes artifacts; CI uploads the output directory.

**Tech Stack:** Rust 2024 edition, serde JSON, text rendering tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M29-04 spec and M29-02 writer code**

### Task 2: Write Tests

- [x] **Step 1: Add artifact path and summary tests**

### Task 3: Implement

- [x] **Step 1: Add stable artifact set contract**
- [x] **Step 2: Ensure partial reports are written on failure**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused artifact checks**

```bash
cargo fmt --all -- --check
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask quality_gate
```

```bash
git add xtask docs
git commit -m "feat: stabilize quality gate artifacts"
```
