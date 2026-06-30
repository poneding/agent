# M47-00 MCP HTTP Session Resume Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M47 architecture baseline and implementation slice map.

**Architecture:** `agent-mcp` owns HTTP session metadata, response envelopes, session-aware headers, and SSE checkpoints; CLI/Desktop remain presentation clients.

**Tech Stack:** Markdown specs/plans, existing Rust workspace.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M45-mcp-client-streamable-http.spec.md`
- Read: `docs/specs/M46-mcp-http-sse-response-runtime.spec.md`
- Read: `crates/agent-mcp/src/client_http.rs`

- [x] **Step 1: Review completed HTTP and SSE client shape**

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M47-00-mcp-http-session-resume-technical-design.spec.md`

- [x] **Step 1: Define ownership, contracts, state, security, and tests**

### Task 3: Commit Planning Baseline

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M47-mcp-http-session-resume-runtime.plan.md`

- [x] **Step 1: Include M47-00 in the planning commit**
