# M49-00 MCP HTTP Stream Reconnect Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the M49 reconnect implementation boundary before runtime changes begin.

**Architecture:** Documentation-only design baseline for `agent-mcp` reconnect policy, runner, replay diagnostics, trace reports, redaction, and verification.

**Tech Stack:** Markdown, Mermaid, Rust contract sketches.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M48-mcp-http-stream-ownership-runtime.spec.md`
- Read: `docs/plans/M48-mcp-http-stream-ownership-runtime.plan.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review M48 boundary and current stream state**

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M49-00-mcp-http-stream-reconnect-technical-design.spec.md`

- [x] **Step 1: Define M49 ownership, contracts, state, failures, and tests**

### Task 3: Decompose Slices

**Files:**
- Create: `docs/specs/M49-01-reconnect-policy-contracts.spec.md`
- Create: `docs/specs/M49-02-stream-reconnect-runner.spec.md`
- Create: `docs/specs/M49-03-missed-event-replay-diagnostics.spec.md`
- Create: `docs/specs/M49-04-reconnect-trace-report.spec.md`
- Create: `docs/specs/M49-05-hardening-docs.spec.md`

- [x] **Step 1: Define M49 slice boundaries**

### Task 4: Update Milestone Index

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Add M49 milestone and active work references**

### Task 5: Commit

- [x] **Step 1: Commit M49 planning**

Run:

```bash
git add docs
git commit -m "docs: plan m49 mcp http stream reconnect runtime"
```
