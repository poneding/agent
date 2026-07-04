# M31-02 Export Profile Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit MCP server export profile bootstrap.

**Architecture:** `agent-mcp::server_profile` owns profile validation and registry construction.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M31-02 spec and M30 export registries**

### Task 2: Write Tests

- [x] **Step 1: Add export profile tests**

### Task 3: Implement

- [x] **Step 1: Add profile contracts**
- [x] **Step 2: Add default-deny bootstrap**
- [x] **Step 3: Add fixture profile construction**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_profile
```

Verified with:

```bash
CARGO_TARGET_DIR=target/codex-m31 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m31 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp mcp_server_profile
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp server export profiles"
```
