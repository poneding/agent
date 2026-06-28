# M28-03 Golden Update Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit golden update workflow contracts.

**Architecture:** `agent-eval::golden` owns update logic; CLI only invokes and renders.

**Tech Stack:** Rust 2024 edition, serde_json, existing golden comparison contracts.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M28-03 spec and current `agent-eval::golden`**

### Task 2: Write Tests

- [x] **Step 1: Add dry-run, write, malformed, and redaction tests**

### Task 3: Implement

- [x] **Step 1: Add update contracts**
- [x] **Step 2: Add suite-aware update writer**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-eval --all-targets -- -D warnings
cargo test -p agent-eval golden_update
```
