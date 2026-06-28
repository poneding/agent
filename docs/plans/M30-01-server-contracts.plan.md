# M30-01 Server Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP server protocol contracts in `agent-mcp`.

**Architecture:** `agent-mcp::server` owns serde protocol contracts. No CLI/Desktop/runtime orchestration code changes in this slice.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M30-01 spec and current `agent-mcp` error/config patterns**

### Task 2: Write Tests

- [x] **Step 1: Add stable protocol contract tests**

### Task 3: Implement

- [x] **Step 1: Add `server.rs` contracts**
- [x] **Step 2: Export server contracts from `agent_mcp`**
- [x] **Step 3: Add validation and redaction**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused server contract checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_contracts
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp server contracts"
```
