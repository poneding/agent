# M34-02 CLI Profile Inspect Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `agent-cli mcp server profile inspect`.

**Architecture:** `agent-cli` parses and renders; `agent-mcp` inspects.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M34-02 spec and CLI MCP command parser**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-cli/tests/mcp_server_profile_inspect_command.rs`

- [x] **Step 1: Add parser test**
- [x] **Step 2: Add text renderer test**
- [x] **Step 3: Add JSON renderer test**

### Task 3: Implement Command

**Files:**
- Modify: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Add command enum variant**
- [x] **Step 2: Add parser and renderer**
- [x] **Step 3: Wire manifest IO and inspection call**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command
```

- [x] **Step 1: Run focused CLI checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m34 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m34 cargo test -p agent-cli mcp_server_profile_inspect_command
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp profile inspect command"
```

- [x] **Step 1: Commit M34-02**
