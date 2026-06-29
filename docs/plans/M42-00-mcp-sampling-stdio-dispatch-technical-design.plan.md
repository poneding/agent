# M42-00 MCP Sampling Stdio Dispatch Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish M42 crate ownership, dispatcher boundaries, and implementation slices.

**Architecture:** `agent-mcp` owns sampling stdio dispatch; `agent-core` remains the provider bridge owner.

**Tech Stack:** Markdown docs, Rust crate boundary review.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-mcp-sampling-model-bridge.spec.md`
- Read: `crates/agent-mcp/src/client.rs`
- Read: `crates/agent-mcp/src/server.rs`
- Read: `crates/agent-mcp/src/server_stdio.rs`

- [x] **Step 1: Review prior sampling and JSON-RPC contracts**

Expected: M42 can reuse existing sampling contracts, provider trait, config, and JSON-RPC envelopes.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M42-00-mcp-sampling-stdio-dispatch-technical-design.spec.md`
- Create: `docs/plans/M42-00-mcp-sampling-stdio-dispatch-technical-design.plan.md`

- [x] **Step 1: Document ownership and boundaries**
- [x] **Step 2: Define slice decomposition**

### Task 3: Verify

- [x] **Step 1: Run docs whitespace check**

Run:

```bash
git diff --check
```

Expected: no whitespace errors.
