# M34-04 JSON Redaction Regressions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable JSON and redaction coverage for profile inspection.

**Architecture:** `ruse-mcp` produces safe data; `ruse-cli` renders it unchanged.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M34 inspection and CLI output**

### Task 2: Add Regression Tests

- [x] **Step 1: Add JSON shape assertions**
- [x] **Step 2: Add secret-like value redaction assertions**
- [x] **Step 3: Add protocol-only stdio regression if needed**

### Task 3: Implement Fixes

- [x] **Step 1: Adjust redaction or rendering gaps**

No implementation changes were needed beyond regression coverage; existing redaction and protocol-only stdio behavior already satisfied this slice.

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_inspection
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-cli mcp_server_profile_inspect_command
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p ruse-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-mcp mcp_server_profile_inspection
CARGO_TARGET_DIR=target/codex-m34 cargo test -p ruse-cli mcp_server_profile_inspect_command
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: harden mcp profile inspect redaction"
```

- [x] **Step 1: Commit M34-04**
