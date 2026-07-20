# M33-01 Built-in Tool Export Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a public helper that builds the curated MCP built-in tool export catalog from a workspace sandbox.

**Architecture:** `ruse-mcp` constructs `McpServerToolExportCatalog` from selected `ruse-tool` built-ins and `ruse-sandbox`.

**Tech Stack:** Rust 2024 edition, Cargo tests.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M33-01-builtin-tool-export-catalog.spec.md`
- Read: `crates/ruse-mcp/src/server_manifest.rs`
- Read: `crates/ruse-tool/src/builtins/mod.rs`

- [x] **Step 1: Review catalog and built-in tool contracts**

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_server_builtin_tool_catalog.rs`

- [x] **Step 1: Test allow-list names**
- [x] **Step 2: Test side-effecting names are absent**
- [x] **Step 3: Test invalid workspace root failure**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_catalog
```

Expected: tests fail before implementation.

### Task 3: Implement Catalog Helper

**Files:**
- Modify: `crates/ruse-mcp/Cargo.toml`
- Modify: `crates/ruse-mcp/src/server_manifest.rs`
- Modify: `crates/ruse-mcp/src/lib.rs`

- [x] **Step 1: Add `ruse-sandbox` dependency if needed**
- [x] **Step 2: Add config type and builder function**
- [x] **Step 3: Register only read/search/git inspection tools**

### Task 4: Verify

- [x] **Step 1: Run focused tests**

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m33 cargo clippy -p ruse-mcp --all-targets -- -D warnings
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_catalog
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp
```

Verified:

```text
fmt-check passed
clippy -p ruse-mcp passed
ruse-mcp mcp_server_builtin_tool_catalog: 4 passed
ruse-mcp full crate tests passed
```

### Task 5: Commit

- [x] **Step 1: Update progress and commit**

```bash
git add Cargo.toml Cargo.lock crates docs
git commit -m "feat: add mcp builtin tool export catalog"
```
