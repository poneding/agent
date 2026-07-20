# M28-04 CLI Eval Suite Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI suite commands over `ruse-eval` suite contracts.

**Architecture:** CLI renders typed suite reports; runtime logic remains in `ruse-eval`.

**Tech Stack:** Rust 2024 edition, existing CLI parser style, serde_json, tempfile tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M28-04 spec and current eval CLI commands**

### Task 2: Write Tests

- [x] **Step 1: Add real binary tests for suite list/validate/run/update**

### Task 3: Implement

- [x] **Step 1: Add parser variants**
- [x] **Step 2: Delegate to `ruse-eval` suite APIs**
- [x] **Step 3: Render text and JSON**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-cli --all-targets -- -D warnings
cargo test -p ruse-cli --test eval_suite_commands
```
