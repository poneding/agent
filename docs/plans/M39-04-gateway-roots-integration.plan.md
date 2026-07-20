# M39-04 Gateway Roots Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe root metadata and diagnostics to MCP gateway reports.

**Architecture:** `ruse-mcp::gateway` consumes client root provider APIs and keeps policy/tool behavior unchanged.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo tests.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Review M39-04 spec and existing gateway report tests**

### Task 2: Write Failing Tests

- [x] **Step 1: Add healthy server root report tests**
- [x] **Step 2: Add disabled/unhealthy server exclusion tests**
- [x] **Step 3: Add provider error redaction diagnostics tests**
- [x] **Step 4: Add unchanged tool registration regression tests**

### Task 3: Implement Gateway Integration

- [x] **Step 1: Add root descriptor or summary report fields**
- [x] **Step 2: Query roots after health success**
- [x] **Step 3: Add deterministic sorting and diagnostics**

### Task 4: Verify

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p ruse-mcp mcp_gateway_roots
```

- [x] **Step 1: Run focused checks**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m39 cargo test -p ruse-mcp mcp_gateway_roots
```

### Task 5: Commit

```bash
git add crates docs
git commit -m "feat: add mcp gateway roots diagnostics"
```

- [x] **Step 1: Commit M39-04**
