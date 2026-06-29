# M31-04 Server Trace Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add trace-safe MCP server session summaries.

**Architecture:** `agent-mcp::server_trace` owns trace contracts; sessions emit summaries without CLI ownership of redaction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M31-04 spec and existing observability redaction patterns**

### Task 2: Write Tests

- [x] **Step 1: Add server trace tests**

### Task 3: Implement

- [x] **Step 1: Add trace contracts**
- [x] **Step 2: Integrate trace emission with session runtime**
- [x] **Step 3: Add bounded summary rendering**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m31 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m31 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp mcp_server_trace
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp
```

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_trace
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp server trace summaries"
```
