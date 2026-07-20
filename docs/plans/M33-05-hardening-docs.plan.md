# M33-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M33 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M33 specs and completed implementation slices**

### Task 2: Update Docs

- [x] **Step 1: Update README built-in tool export usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M33 focused tests**
- [x] **Step 2: Run `cargo xtask ci`**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_catalog
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_manifest_tool_diagnostics
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-mcp mcp_server_builtin_tool_call
CARGO_TARGET_DIR=target/codex-m33 cargo test -p ruse-cli mcp_server_workspace_tool_profile
CARGO_TARGET_DIR=target/codex-m33 cargo xtask ci
```

Quality gate summary:

```text
Quality gate ci-local status=passed
total=4 passed=4 failed=0 skipped=0
fmt-check status=passed
clippy status=passed
test status=passed
eval status=passed
```

### Task 4: Commit

```bash
git add README.md docs crates
git commit -m "feat: complete m33 mcp builtin tool export runtime"
```
