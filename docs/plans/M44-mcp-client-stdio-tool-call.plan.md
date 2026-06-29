# M44 MCP Client Stdio Tool Call Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add live MCP `tools/call` execution over client-side stdio sessions and the existing gateway tool path.

**Architecture:** `agent-mcp` owns JSON-RPC `tools/call` mapping, session use, gateway client implementation, redacted diagnostics, and focused tests. CLI/Desktop remain consumers of typed runtime contracts.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Tokio process and IO, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M44 Technical Design

**Files:**
- Create: `docs/specs/M44-00-mcp-client-stdio-tool-call-technical-design.spec.md`
- Create: `docs/plans/M44-00-mcp-client-stdio-tool-call-technical-design.plan.md`

- [x] **Step 1: Write the M44 technical design baseline**

Define trait ownership, `tools/call` request/result mapping, gateway policy boundary, process E2E shape, redaction, and verification gates.

### Task 2: Decompose M44 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M44-01-stdio-tool-call-mapping.spec.md`
- Create: `docs/plans/M44-01-stdio-tool-call-mapping.plan.md`
- Create: `docs/specs/M44-02-gateway-live-stdio-tools.spec.md`
- Create: `docs/plans/M44-02-gateway-live-stdio-tools.plan.md`
- Create: `docs/specs/M44-03-tool-call-redaction-limits.spec.md`
- Create: `docs/plans/M44-03-tool-call-redaction-limits.plan.md`
- Create: `docs/specs/M44-04-process-e2e.spec.md`
- Create: `docs/plans/M44-04-process-e2e.plan.md`
- Create: `docs/specs/M44-05-hardening-docs.spec.md`
- Create: `docs/plans/M44-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M44 Slices in Order

**Files:**
- Execute: `docs/plans/M44-01-stdio-tool-call-mapping.plan.md`
- Execute: `docs/plans/M44-02-gateway-live-stdio-tools.plan.md`
- Execute: `docs/plans/M44-03-tool-call-redaction-limits.plan.md`
- Execute: `docs/plans/M44-04-process-e2e.plan.md`
- Execute: `docs/plans/M44-05-hardening-docs.plan.md`

- [x] **Step 1: Add stdio `tools/call` mapping**
- [x] **Step 2: Add gateway live stdio execution coverage**
- [ ] **Step 3: Harden tool-call redaction and limits**
- [ ] **Step 4: Add subprocess E2E coverage**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M44

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M44-mcp-client-stdio-tool-call.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m44 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M44 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m44 mcp client stdio tool call"
```
