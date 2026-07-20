# M51-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M51 MCP HTTP OAuth authorization foundation.

**Architecture:** Documentation-only slice after runtime implementation. README and milestone status must reflect the exact implemented boundary.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M51-mcp-http-oauth-authorization-runtime.spec.md`
- Read: `docs/plans/M51-mcp-http-oauth-authorization-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M51 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M51-mcp-http-oauth-authorization-runtime.plan.md`

- [x] **Step 1: Update README authorization usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M51 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_metadata
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_authorization_diagnostics
CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_bearer_authorization
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m51 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M51-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m51 mcp http oauth authorization runtime"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_metadata`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_oauth_discovery`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_authorization_diagnostics`
- `CARGO_TARGET_DIR=target/codex-m51 cargo test -p ruse-mcp mcp_client_http_bearer_authorization`
- `git diff --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo fmt --all -- --check`
- `CARGO_TARGET_DIR=target/codex-m51 cargo xtask ci`

Note: `cargo xtask ci` printed the expected fake rustc flag fixture message, `error: Unrecognized option: 'definitely-not-a-real-rustc-flag'`, and exited successfully.
