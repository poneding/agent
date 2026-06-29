# M31-03 CLI Stdio Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `agent-cli mcp server stdio` over the shared MCP server session runtime.

**Architecture:** `agent-cli` parses command and wires IO; `agent-mcp` owns protocol/session semantics.

**Tech Stack:** Rust 2024 edition, stdio, subprocess tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M31-03 spec and existing CLI command patterns**

### Task 2: Write Tests

- [ ] **Step 1: Add CLI parse and subprocess tests**

### Task 3: Implement

- [ ] **Step 1: Extend `McpServerCommand`**
- [ ] **Step 2: Add stdio command runner**
- [ ] **Step 3: Preserve protocol-only stdout**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli --all-targets -- -D warnings
cargo test -p agent-cli mcp_server_stdio_command
```

```bash
git add crates/agent-cli docs
git commit -m "feat: add mcp server stdio command"
```
