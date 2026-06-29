# M34-04 JSON Redaction Regressions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable JSON and redaction coverage for profile inspection.

**Architecture:** `agent-mcp` produces safe data; `agent-cli` renders it unchanged.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M34 inspection and CLI output**

### Task 2: Add Regression Tests

- [ ] **Step 1: Add JSON shape assertions**
- [ ] **Step 2: Add secret-like value redaction assertions**
- [ ] **Step 3: Add protocol-only stdio regression if needed**

### Task 3: Implement Fixes

- [ ] **Step 1: Adjust redaction or rendering gaps**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-mcp mcp_server_profile_inspection
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: harden mcp profile inspect redaction"
```

- [ ] **Step 1: Commit M34-04**
