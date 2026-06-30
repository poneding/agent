# M50-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M50 MCP HTTP stream supervision foundation.

**Architecture:** Documentation-only slice after runtime implementation. README and milestone status must reflect the exact implemented boundary.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M50-mcp-http-background-stream-supervision.spec.md`
- Read: `docs/plans/M50-mcp-http-background-stream-supervision.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M50 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M50-mcp-http-background-stream-supervision.plan.md`

- [x] **Step 1: Update README supervision usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M50 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_contracts
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_runner
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_cancellation
CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_reports
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m50 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M50-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m50 mcp http background stream supervision"
```

### Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_runner` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_cancellation` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo test -p agent-mcp mcp_client_http_stream_supervisor_reports` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m50 cargo xtask ci` passed. The quality-gate fixture printed the expected fake rustc flag error and exited successfully.
