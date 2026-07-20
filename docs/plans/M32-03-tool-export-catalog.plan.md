# M32-03 Tool Export Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register manifest-declared MCP tools from an explicit catalog.

**Architecture:** `ruse-mcp` performs catalog validation and profile building; callers provide concrete tools.

**Tech Stack:** Rust 2024 edition, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M32-03 spec and tool registry behavior**

### Task 2: Write Tests

- [x] **Step 1: Add tool catalog tests**

### Task 3: Implement

- [x] **Step 1: Add catalog contract**
- [x] **Step 2: Bind manifest declarations to catalog tools**
- [x] **Step 3: Preserve policy-gated execution**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m32 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m32 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp mcp_server_tool_catalog
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp
```

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-mcp --all-targets -- -D warnings
cargo test -p ruse-mcp mcp_server_tool_catalog
```

```bash
git add crates/ruse-mcp docs
git commit -m "feat: add mcp tool export catalog"
```
