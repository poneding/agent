# M29-02 Xtask CI Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `cargo xtask ci` with quality gate execution and evidence output.

**Architecture:** `xtask` plans and runs commands; runtime crates stay unaware of CI.

**Tech Stack:** Rust 2024 edition, std::process, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M29-02 spec and M29-01 contracts**

### Task 2: Write Tests

- [x] **Step 1: Add parser, plan, artifact, and failure tests**

### Task 3: Implement

- [x] **Step 1: Add `Task::Ci` and help text**
- [x] **Step 2: Add quality gate runner**
- [x] **Step 3: Write JSON and text artifacts**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused xtask checks**
- [x] **Step 2: Run full local quality gate**

```bash
cargo fmt --all -- --check
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask quality_gate
cargo test -p xtask --test xtask_plan
cargo xtask ci
```

```bash
git add xtask docs
git commit -m "feat: add xtask ci quality gate"
```
