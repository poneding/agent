# M28-02 Built-in Eval Corpus Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a checked-in offline evaluation corpus.

**Architecture:** `docs/eval` stores data; `agent-eval` loads and runs it through suite contracts.

**Tech Stack:** JSON fixtures, existing M27/M28 eval contracts, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M28-02 spec and M28-01 suite contracts**

### Task 2: Add Corpus Files

- [x] **Step 1: Create default suite and scenarios**
- [x] **Step 2: Add deterministic fixtures and goldens**

### Task 3: Add Tests

- [x] **Step 1: Add corpus load/run/golden smoke tests**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-eval --all-targets -- -D warnings
cargo test -p agent-eval corpus
```

- [x] **Step 2: Commit**

```bash
git add crates/agent-eval docs
git commit -m "feat: add builtin eval corpus"
```
