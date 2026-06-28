# M28-05 Xtask Eval Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `cargo xtask eval` as a CI-friendly evaluation gate.

**Architecture:** `xtask` invokes CLI suite commands and does not own eval parsing.

**Tech Stack:** Rust 2024 edition, existing xtask parser/plan tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M28-05 spec and `xtask/src/lib.rs`**

### Task 2: Write Tests

- [ ] **Step 1: Add xtask eval parser and command plan tests**

### Task 3: Implement

- [ ] **Step 1: Add `Task::Eval`**
- [ ] **Step 2: Add command plan and help text**

### Task 4: Complete M28

- [ ] **Step 1: Update README and milestone status**
- [ ] **Step 2: Run final checks**

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md xtask
git commit -m "feat: complete m28 evaluation quality gates"
```
