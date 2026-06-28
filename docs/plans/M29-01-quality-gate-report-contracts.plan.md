# M29-01 Quality Gate Report Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed quality gate evidence report contracts to `xtask`.

**Architecture:** `xtask` owns quality gate contracts. No runtime crate should depend on CI concepts.

**Tech Stack:** Rust 2024 edition, serde, existing xtask tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M29-01 spec and existing xtask command plan code**

### Task 2: Write Tests

- [ ] **Step 1: Add report serde, aggregation, summary, and redaction tests**

### Task 3: Implement

- [ ] **Step 1: Add quality gate structs and status aggregation**
- [ ] **Step 2: Add summary renderer and redaction helper**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask quality_gate
```

```bash
git add xtask docs
git commit -m "feat: add quality gate report contracts"
```
