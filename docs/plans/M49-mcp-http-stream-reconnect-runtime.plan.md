# M49 MCP HTTP Stream Reconnect Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bounded MCP streamable HTTP reconnect behavior with safe retry, resume, replay diagnostics, and redacted reports.

**Architecture:** `agent-mcp` owns reconnect policy, runner, replay diagnostics, trace reports, and tests. CLI/Desktop consume typed reports only.

**Tech Stack:** Rust 2024 edition, async-trait, serde, reqwest fixture transports, SSE parsing, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M49 Technical Design

**Files:**
- Create: `docs/specs/M49-00-mcp-http-stream-reconnect-technical-design.spec.md`
- Create: `docs/plans/M49-00-mcp-http-stream-reconnect-technical-design.plan.md`

- [x] **Step 1: Write the M49 technical design baseline**

Define crate ownership, reconnect policy, runner composition, replay diagnostics, trace reports, redaction behavior, and verification gates.

### Task 2: Decompose M49 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M49-01-reconnect-policy-contracts.spec.md`
- Create: `docs/plans/M49-01-reconnect-policy-contracts.plan.md`
- Create: `docs/specs/M49-02-stream-reconnect-runner.spec.md`
- Create: `docs/plans/M49-02-stream-reconnect-runner.plan.md`
- Create: `docs/specs/M49-03-missed-event-replay-diagnostics.spec.md`
- Create: `docs/plans/M49-03-missed-event-replay-diagnostics.plan.md`
- Create: `docs/specs/M49-04-reconnect-trace-report.spec.md`
- Create: `docs/plans/M49-04-reconnect-trace-report.plan.md`
- Create: `docs/specs/M49-05-hardening-docs.spec.md`
- Create: `docs/plans/M49-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M49 Slices in Order

**Files:**
- Execute: `docs/plans/M49-01-reconnect-policy-contracts.plan.md`
- Execute: `docs/plans/M49-02-stream-reconnect-runner.plan.md`
- Execute: `docs/plans/M49-03-missed-event-replay-diagnostics.plan.md`
- Execute: `docs/plans/M49-04-reconnect-trace-report.plan.md`
- Execute: `docs/plans/M49-05-hardening-docs.plan.md`

- [x] **Step 1: Add reconnect policy contracts**
- [x] **Step 2: Add bounded stream reconnect runner**
- [x] **Step 3: Add missed-event replay diagnostics**
- [x] **Step 4: Add reconnect trace reports**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M49

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M49-mcp-http-stream-reconnect-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m49 cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark M49 complete and commit**

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
