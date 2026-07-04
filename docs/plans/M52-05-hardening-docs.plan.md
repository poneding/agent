# M52-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M52 MCP HTTP OAuth token lifecycle foundation.

**Architecture:** Documentation-only slice after runtime implementation.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M52-mcp-http-oauth-token-lifecycle-runtime.spec.md`
- Read: `docs/plans/M52-mcp-http-oauth-token-lifecycle-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M52 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M52-mcp-http-oauth-token-lifecycle-runtime.plan.md`

- [x] **Step 1: Update README token lifecycle usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M52 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_source
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_refresh
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_requests
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m52 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M52-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m52 mcp http oauth token lifecycle runtime"
```

Verification evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_source
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_refresh
CARGO_TARGET_DIR=target/codex-m52 cargo test -p agent-mcp mcp_client_http_oauth_token_requests
git diff --check
CARGO_TARGET_DIR=target/codex-m52 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m52 cargo xtask ci
```

Result: all final M52 checks passed. `cargo xtask ci` prints the expected `definitely-not-a-real-rustc-flag` fixture error text and exits successfully.
