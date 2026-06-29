# M39-02 Client Root Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add validated, deterministic MCP client root registry behavior.

**Architecture:** `agent-mcp` owns root validation and inspection. No CLI/Desktop or filesystem-content logic belongs in this slice.

**Tech Stack:** Rust 2024 edition, path validation, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [ ] **Step 1: Review M39-02 spec and M39-01 contracts**

### Task 2: Write Failing Tests

- [ ] **Step 1: Add valid root registration tests**
- [ ] **Step 2: Add duplicate, disabled, and ordering tests**
- [ ] **Step 3: Add invalid path, URI, traversal, and secret diagnostics tests**

### Task 3: Implement Registry

- [ ] **Step 1: Add root candidate and registry types**
- [ ] **Step 2: Add validation and deterministic list result generation**
- [ ] **Step 3: Add inspection diagnostics**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p agent-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_root_registry
```

- [ ] **Step 1: Run focused checks**

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp client root registry"
```

- [ ] **Step 1: Commit M39-02**
