# M32-03 Tool Export Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register manifest-declared MCP tools from an explicit catalog.

**Architecture:** `agent-mcp` performs catalog validation and profile building; callers provide concrete tools.

**Tech Stack:** Rust 2024 edition, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M32-03 spec and tool registry behavior**

### Task 2: Write Tests

- [ ] **Step 1: Add tool catalog tests**

### Task 3: Implement

- [ ] **Step 1: Add catalog contract**
- [ ] **Step 2: Bind manifest declarations to catalog tools**
- [ ] **Step 3: Preserve policy-gated execution**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_tool_catalog
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp tool export catalog"
```
