# M49-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document and verify the completed M49 MCP HTTP stream reconnect foundation.

**Architecture:** Documentation-only slice after runtime implementation. README and milestone status must reflect the exact implemented boundary.

**Tech Stack:** Markdown, Cargo fmt, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M49-mcp-http-stream-reconnect-runtime.spec.md`
- Read: `docs/plans/M49-mcp-http-stream-reconnect-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M49 specs and completed implementation slices**

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M49-mcp-http-stream-reconnect-runtime.plan.md`

- [x] **Step 1: Update README reconnect usage**
- [x] **Step 2: Update milestone status and latest docs links**

### Task 3: Verify

- [x] **Step 1: Run final M49 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_policy
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_stream_reconnect_runner
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_missed_event_replay
CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_trace
```

- [x] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m49 cargo xtask ci
```

### Task 4: Commit

- [x] **Step 1: Commit M49-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m49 mcp http stream reconnect runtime"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_policy` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_stream_reconnect_runner` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_missed_event_replay` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo test -p agent-mcp mcp_client_http_reconnect_trace` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m49 cargo xtask ci` passed. The known quality-gate fixture printed `error: Unrecognized option: 'definitely-not-a-real-rustc-flag'` and the command exited 0.
