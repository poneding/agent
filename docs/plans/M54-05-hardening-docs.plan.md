# M54-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M54 MCP HTTP OAuth token exchange runtime.

**Architecture:** Documentation-only slice after runtime implementation.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M54-mcp-http-oauth-token-exchange-runtime.spec.md`
- Read: `docs/plans/M54-mcp-http-oauth-token-exchange-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M54 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M54-mcp-http-oauth-token-exchange-runtime.plan.md`

- [x] **Step 1: Update README token exchange usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M54 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_response
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_transport
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_runner
CARGO_TARGET_DIR=target/codex-m54 cargo test -p ruse-mcp mcp_client_http_oauth_token_exchange_lease
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m54 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m54 cargo xtask ci
```

### Task 4: Commit

- [ ] **Step 1: Commit M54-05**

Run:

```bash
git add README.md docs crates Cargo.toml Cargo.lock
git commit -m "feat: complete m54 mcp http oauth token exchange runtime"
```
