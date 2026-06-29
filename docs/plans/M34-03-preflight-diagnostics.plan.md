# M34-03 Preflight Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add profile inspection diagnostics that predict stdio startup failures.

**Architecture:** `agent-mcp` owns diagnostics; CLI renders them.

**Tech Stack:** Rust 2024 edition, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M34-03 spec and M34-01 inspection contracts**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add missing workspace and invalid workspace tests**
- [ ] **Step 2: Add unknown tool and missing resource tests**
- [ ] **Step 3: Add CLI rendering coverage**

### Task 3: Implement Diagnostics

- [ ] **Step 1: Add diagnostic codes and severities**
- [ ] **Step 2: Align diagnostics with stdio startup behavior**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-mcp mcp_server_profile_preflight_diagnostics
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp profile preflight diagnostics"
```

- [ ] **Step 1: Commit M34-03**
