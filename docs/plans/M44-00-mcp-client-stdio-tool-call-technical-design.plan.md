# M44-00 MCP Client Stdio Tool Call Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M44 implementation boundary for live MCP stdio `tools/call`.

**Architecture:** `agent-mcp` owns protocol mapping and gateway client behavior; UI crates remain presentation-only.

**Tech Stack:** Markdown specs/plans, Rust workspace context.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M43-mcp-client-stdio-transport.spec.md`
- Read: `crates/agent-mcp/src/client_stdio.rs`
- Read: `crates/agent-mcp/src/gateway.rs`
- Read: `crates/agent-mcp/src/tool.rs`

- [x] **Step 1: Review M43 stdio client and gateway boundaries**

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M44-00-mcp-client-stdio-tool-call-technical-design.spec.md`

- [x] **Step 1: Document ownership, contracts, state, privacy, and tests**

### Task 3: Commit

- [x] **Step 1: Include design in M44 planning commit**
