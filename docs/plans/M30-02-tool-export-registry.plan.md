# M30-02 Tool Export Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a policy-aware local tool export registry for MCP server mode.

**Architecture:** `agent-mcp` maps `agent-tool` definitions into MCP server descriptors and uses `agent-policy` before execution.

**Tech Stack:** Rust 2024 edition, serde JSON, existing `agent-tool` and `agent-policy` crates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M30-02 spec and tool registry contracts**

### Task 2: Write Tests

- [x] **Step 1: Add tool export registry tests**

### Task 3: Implement

- [x] **Step 1: Add `server_tool.rs`**
- [x] **Step 2: Convert local tool definitions into MCP tool descriptors**
- [x] **Step 3: Add policy-aware fixture calls**

### Task 4: Verify and Commit

- [x] **Step 1: Run focused tool export checks**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_tool_export
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp server tool exports"
```
