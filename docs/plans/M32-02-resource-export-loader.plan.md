# M32-02 Resource Export Loader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build MCP resource exports from manifest resource declarations.

**Architecture:** `agent-mcp` owns file loading and profile construction; CLI remains path plumbing.

**Tech Stack:** Rust 2024 edition, std fs/path APIs, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Read M32-02 spec and resource registry behavior**

### Task 2: Write Tests

- [ ] **Step 1: Add resource loader tests**

### Task 3: Implement

- [ ] **Step 1: Add resource loading helper**
- [ ] **Step 2: Integrate helper with profile builder**
- [ ] **Step 3: Add safe diagnostics**

### Task 4: Verify and Commit

```bash
cargo fmt --all -- --check
cargo clippy -p agent-mcp --all-targets -- -D warnings
cargo test -p agent-mcp mcp_server_resource_loader
```

```bash
git add crates/agent-mcp docs
git commit -m "feat: load mcp resource export profiles"
```
