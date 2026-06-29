# M31-01 Stdio Session Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add reusable MCP stdio session runtime contracts to `agent-mcp`.

**Architecture:** `agent-mcp::server_session` owns session state and delegates frame dispatch to M30 server code.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M31-01 spec and M30 dispatcher contracts**

### Task 2: Write Tests

- [ ] **Step 1: Add session lifecycle tests**

### Task 3: Implement

- [ ] **Step 1: Add `server_session.rs`**
- [ ] **Step 2: Enforce frame and session limits**
- [ ] **Step 3: Export contracts from `agent-mcp`**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_session
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp stdio session runtime"
```
