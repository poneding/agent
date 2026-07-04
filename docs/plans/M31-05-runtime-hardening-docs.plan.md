# M31-05 Runtime Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M31 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice. Runtime ownership remains in `agent-mcp`; CLI remains an IO shell.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M31 specs and completed implementation slices**

### Task 2: Update Docs

- [x] **Step 1: Update README MCP stdio server usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M31 focused tests**
- [x] **Step 2: Run `cargo xtask ci`**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp mcp_server_session
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp mcp_server_profile
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-mcp mcp_server_trace
CARGO_TARGET_DIR=target/codex-m31 cargo test -p agent-cli mcp_server_stdio_command
CARGO_TARGET_DIR=target/codex-m31 cargo xtask ci
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
git commit -m "feat: complete m31 mcp stdio server runtime"
```
