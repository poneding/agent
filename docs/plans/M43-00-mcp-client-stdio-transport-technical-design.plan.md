# M43-00 MCP Client Stdio Transport Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M43 technical design and implementation slice map.

**Architecture:** Documentation-only baseline for `ruse-mcp` client stdio transport ownership.

**Tech Stack:** Markdown docs, Mermaid diagrams, Cargo workspace conventions.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-mcp-sampling-stdio-dispatch.spec.md`
- Read: `crates/ruse-mcp/src/client.rs`
- Read: `crates/ruse-mcp/src/server_session.rs`
- Read: `crates/ruse-mcp/src/server_stdio.rs`

- [x] **Step 1: Review prior MCP stdio boundaries**

Expected: M43 follows existing separation between protocol runtime and CLI/process presentation.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M43-00-mcp-client-stdio-transport-technical-design.spec.md`
- Create: `docs/plans/M43-00-mcp-client-stdio-transport-technical-design.plan.md`

- [x] **Step 1: Document ownership, lifecycle, and failure handling**
- [x] **Step 2: Document privacy and testing rules**

### Task 3: Verify

- [x] **Step 1: Check docs are linked from milestone index**

Expected: `docs/milestones/README.md` references M43 technical design.

### Task 4: Commit

- [x] **Step 1: Commit M43 technical design**

Run:

```bash
git add docs
git commit -m "docs: plan m43 mcp client stdio transport"
```
