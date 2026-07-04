# M47-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M47 MCP HTTP session resume foundation.

**Architecture:** Documentation-only slice after runtime implementation. README and milestone status must reflect the exact implemented boundary.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-mcp-http-session-resume-runtime.spec.md`
- Read: `docs/plans/M47-mcp-http-session-resume-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M47 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M47-mcp-http-session-resume-runtime.plan.md`

- [x] **Step 1: Update README session metadata usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M47 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_metadata
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_response_envelope
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_headers
CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_sse_checkpoints
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m47 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M47-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m47 mcp http session resume runtime"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_metadata` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_response_envelope` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_session_headers` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo test -p agent-mcp mcp_client_http_sse_checkpoints` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m47 cargo xtask ci` passed. The `definitely-not-a-real-rustc-flag` output is expected from an existing quality-gate fixture.
