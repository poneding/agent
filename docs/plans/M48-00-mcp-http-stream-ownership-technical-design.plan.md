# M48-00 MCP HTTP Stream Ownership Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the M48 stream ownership implementation boundary before runtime changes begin.

**Architecture:** Documentation-only design baseline for `agent-mcp` GET stream contracts, transport, resume headers, drain, redaction, and verification.

**Tech Stack:** Markdown, Mermaid, Rust contract sketches.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M47-mcp-http-session-resume-runtime.spec.md`
- Read: `docs/plans/M47-mcp-http-session-resume-runtime.plan.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review M47 boundary and current HTTP client state**

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M48-00-mcp-http-stream-ownership-technical-design.spec.md`

- [x] **Step 1: Define M48 ownership, contracts, state, failures, and tests**

### Task 3: Decompose Slices

**Files:**
- Create: `docs/specs/M48-01-http-stream-request-contracts.spec.md`
- Create: `docs/specs/M48-02-reqwest-get-sse-stream-open.spec.md`
- Create: `docs/specs/M48-03-stream-resume-snapshot.spec.md`
- Create: `docs/specs/M48-04-bounded-stream-event-drain.spec.md`
- Create: `docs/specs/M48-05-hardening-docs.spec.md`

- [x] **Step 1: Define M48 slice boundaries**

### Task 4: Update Milestone Index

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Add M48 milestone and active work references**

### Task 5: Commit

- [ ] **Step 1: Commit M48 planning**

Run:

```bash
git add docs
git commit -m "docs: plan m48 mcp http stream ownership runtime"
```
