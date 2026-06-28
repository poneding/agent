# M30-03 Resource Export Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add read-only MCP server resource exports.

**Architecture:** `agent-mcp::server_resource` owns resource descriptors and bounded reads.

**Tech Stack:** Rust 2024 edition, serde JSON, URL/URI validation through simple deterministic rules.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M30-03 spec and existing redaction patterns**

### Task 2: Write Tests

- [ ] **Step 1: Add resource export tests**

### Task 3: Implement

- [ ] **Step 1: Add `server_resource.rs`**
- [ ] **Step 2: Add deterministic list/read behavior**
- [ ] **Step 3: Add bounding and diagnostics**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_resource_export
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: add mcp server resource exports"
```
