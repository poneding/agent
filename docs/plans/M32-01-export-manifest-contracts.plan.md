# M32-01 Export Manifest Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP server export manifest contracts in `agent-mcp`.

**Architecture:** `agent-mcp::server_manifest` owns serde and validation; no CLI or file loading in this slice.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M32-01 spec and existing profile validation code**

### Task 2: Write Tests

- [ ] **Step 1: Add manifest serde and validation tests**

### Task 3: Implement

- [ ] **Step 1: Add manifest contracts**
- [ ] **Step 2: Add parse and selection helpers**
- [ ] **Step 3: Export contracts from `agent_mcp`**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_manifest
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp export manifest contracts"
```
