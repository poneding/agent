# M43 MCP Client Stdio Transport Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real client-side MCP stdio transport path for initialize and tools/list.

**Architecture:** `agent-mcp` owns frame codecs, client session state, transport abstraction, spawned process handling, typed result mapping, and redacted diagnostics. CLI/Desktop surfaces consume those contracts later.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Tokio process and IO, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M43 Technical Design

**Files:**
- Create: `docs/specs/M43-00-mcp-client-stdio-transport-technical-design.spec.md`
- Create: `docs/plans/M43-00-mcp-client-stdio-transport-technical-design.plan.md`

- [x] **Step 1: Write the M43 technical design baseline**

Define crate ownership, transport trait boundaries, request/response id handling, process lifecycle, timeout/cancellation, redaction, and verification gates.

### Task 2: Decompose M43 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M43-01-client-stdio-codec.spec.md`
- Create: `docs/plans/M43-01-client-stdio-codec.plan.md`
- Create: `docs/specs/M43-02-client-stdio-session.spec.md`
- Create: `docs/plans/M43-02-client-stdio-session.plan.md`
- Create: `docs/specs/M43-03-client-stdio-process.spec.md`
- Create: `docs/plans/M43-03-client-stdio-process.plan.md`
- Create: `docs/specs/M43-04-client-stdio-mcp-client.spec.md`
- Create: `docs/plans/M43-04-client-stdio-mcp-client.plan.md`
- Create: `docs/specs/M43-05-hardening-docs.spec.md`
- Create: `docs/plans/M43-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M43 Slices in Order

**Files:**
- Execute: `docs/plans/M43-01-client-stdio-codec.plan.md`
- Execute: `docs/plans/M43-02-client-stdio-session.plan.md`
- Execute: `docs/plans/M43-03-client-stdio-process.plan.md`
- Execute: `docs/plans/M43-04-client-stdio-mcp-client.plan.md`
- Execute: `docs/plans/M43-05-hardening-docs.plan.md`

- [x] **Step 1: Add client stdio codec contracts**
- [x] **Step 2: Add deterministic client stdio session runtime**
- [x] **Step 3: Add spawned process transport**
- [ ] **Step 4: Implement `McpClient` over stdio for initialize and tools/list**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M43

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M43-mcp-client-stdio-transport.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m43 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M43 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m43 mcp client stdio transport"
```
