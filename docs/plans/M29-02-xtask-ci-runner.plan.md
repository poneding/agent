# M29-02 Xtask CI Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `cargo xtask ci` with quality gate execution and evidence output.

**Architecture:** `xtask` plans and runs commands; runtime crates stay unaware of CI.

**Tech Stack:** Rust 2024 edition, std::process, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M29-02 spec and M29-01 contracts**

### Task 2: Write Tests

- [ ] **Step 1: Add parser, plan, artifact, and failure tests**

### Task 3: Implement

- [ ] **Step 1: Add `Task::Ci` and help text**
- [ ] **Step 2: Add quality gate runner**
- [ ] **Step 3: Write JSON and text artifacts**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p xtask --all-targets -- -D warnings
cargo test -p xtask quality_gate
```

```bash
git add xtask docs
git commit -m "feat: add xtask ci quality gate"
```
