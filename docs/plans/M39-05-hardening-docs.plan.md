# M39-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M39 documentation, hardening checks, and final milestone status.

**Architecture:** Documentation and final verification slice.

**Tech Stack:** Markdown docs, Cargo workspace quality gates.

---

### Task 1: Confirm Inputs

- [x] **Step 1: Read M39 specs and completed implementation slices**

### Task 2: Update Docs

- [x] **Step 1: Update README client roots usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M39 focused tests**
- [x] **Step 2: Run `cargo xtask ci`**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_root_contracts
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_root_registry
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_client_session_roots
CARGO_TARGET_DIR=target/codex-m39 cargo test -p agent-mcp mcp_gateway_roots
git diff --check
CARGO_TARGET_DIR=target/codex-m39 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m39 cargo xtask ci
```

### Task 4: Commit

```bash
git add README.md docs crates
git commit -m "feat: complete m39 mcp client roots runtime"
```

- [x] **Step 1: Commit M39-05**
