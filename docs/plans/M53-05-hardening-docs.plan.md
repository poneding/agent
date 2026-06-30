# M53-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M53 MCP HTTP OAuth authorization-flow foundation.

**Architecture:** Documentation-only slice after runtime implementation.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M53-mcp-http-oauth-authorization-flow-runtime.spec.md`
- Read: `docs/plans/M53-mcp-http-oauth-authorization-flow-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Read M53 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M53-mcp-http-oauth-authorization-flow-runtime.plan.md`

- [ ] **Step 1: Update README authorization-flow usage**
- [ ] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [ ] **Step 1: Run final M53 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_pkce_state
CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_authorization_request
CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_callback
CARGO_TARGET_DIR=target/codex-m53 cargo test -p agent-mcp mcp_client_http_oauth_flow_diagnostics
```

- [ ] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m53 cargo xtask ci
```

### Task 4: Commit

- [ ] **Step 1: Commit M53-05**

Run:

```bash
git add README.md docs crates Cargo.toml Cargo.lock
git commit -m "feat: complete m53 mcp http oauth authorization flow runtime"
```
