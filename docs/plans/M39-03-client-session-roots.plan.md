# M39-03 Client Session Roots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire MCP client roots into fixture client behavior and capability metadata.

**Architecture:** `agent-mcp::client` depends on root contracts. Gateway and UI crates only consume the resulting typed APIs.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M39-03 spec and current `FixtureMcpClient` behavior**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add fixture roots/list success and empty-list tests**
- [ ] **Step 2: Add unknown server and disabled server tests**
- [ ] **Step 3: Add capability metadata tests**

### Task 3: Implement Client Roots Behavior

- [ ] **Step 1: Add root provider trait or fixture method**
- [ ] **Step 2: Add per-server root storage to `FixtureMcpClient`**
- [ ] **Step 3: Include roots capability metadata in client-facing contracts**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_session_roots
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp client session roots"
```

- [ ] **Step 1: Commit M39-03**
