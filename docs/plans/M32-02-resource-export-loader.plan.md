# M32-02 Resource Export Loader Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build MCP resource exports from manifest resource declarations.

**Architecture:** `ruse-mcp` owns file loading and profile construction; CLI remains path plumbing.

**Tech Stack:** Rust 2024 edition, std fs/path APIs, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M32-02 spec and resource registry behavior**

### Task 2: Write Tests

- [x] **Step 1: Add resource loader tests**

### Task 3: Implement

- [x] **Step 1: Add resource loading helper**
- [x] **Step 2: Integrate helper with profile builder**
- [x] **Step 3: Add safe diagnostics**

### Task 4: Verify and Commit

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m32 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m32 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp mcp_server_resource_loader
CARGO_TARGET_DIR=target/codex-m32 cargo test -p ruse-mcp
```

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-mcp --all-targets -- -D warnings
cargo test -p ruse-mcp mcp_server_resource_loader
```

```bash
git add crates/ruse-mcp docs
git commit -m "feat: load mcp resource export profiles"
```
