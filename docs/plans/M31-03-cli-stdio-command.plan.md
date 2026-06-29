# M31-03 CLI Stdio Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `agent-cli mcp server stdio` over the shared MCP server session runtime.

**Architecture:** `agent-cli` parses command and wires IO; `agent-mcp` owns protocol/session semantics.

**Tech Stack:** Rust 2024 edition, stdio, subprocess tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M31-03 spec and existing CLI command patterns**

### Task 2: Write Tests

- [x] **Step 1: Add CLI parse and subprocess tests**

### Task 3: Implement

- [x] **Step 1: Extend `McpServerCommand`**
- [x] **Step 2: Add stdio command runner**
- [x] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m31 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m31 cargo clippy -p agent-cli --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-cli mcp_server_stdio_command
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-cli
```

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli --all-targets -- -D warnings
cargo test -p agent-cli mcp_server_stdio_command
```

```bash
git add crates/agent-cli docs
git commit -m "feat: add mcp server stdio command"
```
