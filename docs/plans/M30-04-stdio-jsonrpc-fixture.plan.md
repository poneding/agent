# M30-04 Stdio JSON-RPC Fixture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic MCP server JSON-RPC frame handling.

**Architecture:** `ruse-mcp` handles protocol frames; production process lifecycle stays out of this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M30-04 spec and server contracts**

### Task 2: Write Tests

- [x] **Step 1: Add frame handling tests**

### Task 3: Implement

- [x] **Step 1: Add dispatcher**
- [x] **Step 2: Add parse/render helpers**
- [x] **Step 3: Preserve state across malformed frames**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-mcp --all-targets -- -D warnings
cargo test -p ruse-mcp mcp_server_stdio
```

Verified with:

```bash
CARGO_TARGET_DIR=target/codex-m30 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m30 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m30 cargo test -p ruse-mcp mcp_server_stdio
CARGO_TARGET_DIR=target/codex-m30 cargo test -p ruse-mcp
```

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp server stdio fixture"
```
