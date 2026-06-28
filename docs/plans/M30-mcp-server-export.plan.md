# M30 MCP Server Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add policy-aware MCP server mode for selected local tools and resources.

**Architecture:** `agent-mcp` owns protocol/server contracts, `agent-tool` owns executable tool definitions, `agent-policy` owns permission decisions, and UI/CLI crates remain presentation layers.

**Tech Stack:** Rust 2024 edition, serde JSON, async traits where needed, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M30 Technical Design

**Files:**
- Create: `docs/specs/M30-00-mcp-server-technical-design.spec.md`
- Create: `docs/plans/M30-00-mcp-server-technical-design.plan.md`

- [x] **Step 1: Write the M30 technical design baseline**

Define server ownership, protocol contracts, export boundaries, policy routing, privacy, and verification gates.

### Task 2: Decompose M30 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M30-01-server-contracts.spec.md`
- Create: `docs/plans/M30-01-server-contracts.plan.md`
- Create: `docs/specs/M30-02-tool-export-registry.spec.md`
- Create: `docs/plans/M30-02-tool-export-registry.plan.md`
- Create: `docs/specs/M30-03-resource-export-registry.spec.md`
- Create: `docs/plans/M30-03-resource-export-registry.plan.md`
- Create: `docs/specs/M30-04-stdio-jsonrpc-fixture.spec.md`
- Create: `docs/plans/M30-04-stdio-jsonrpc-fixture.plan.md`
- Create: `docs/specs/M30-05-cli-trace-docs.spec.md`
- Create: `docs/plans/M30-05-cli-trace-docs.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines contracts, failure handling, privacy behavior, and tests.

- [x] **Step 2: Create slice plans**

Each plan names files, focused tests, verification commands, and commit points.

### Task 3: Implement M30 Slices in Order

**Files:**
- Execute: `docs/plans/M30-01-server-contracts.plan.md`
- Execute: `docs/plans/M30-02-tool-export-registry.plan.md`
- Execute: `docs/plans/M30-03-resource-export-registry.plan.md`
- Execute: `docs/plans/M30-04-stdio-jsonrpc-fixture.plan.md`
- Execute: `docs/plans/M30-05-cli-trace-docs.plan.md`

- [ ] **Step 1: Add MCP server protocol contracts**

Add initialization, capability, JSON-RPC request/response, and error contracts.

- [ ] **Step 2: Add local tool export registry**

Expose selected local tools through `tools/list` and `tools/call` with policy enforcement.

- [ ] **Step 3: Add resource export registry**

Expose selected read-only resource descriptors and bounded reads.

- [ ] **Step 4: Add stdio JSON-RPC fixture handling**

Handle deterministic line-framed JSON-RPC requests for tests and future server mode.

- [ ] **Step 5: Add CLI, trace, and final docs**

Expose safe server readiness commands, trace summaries, README docs, and final M30 status.

### Task 4: Complete M30

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M30-mcp-server-export.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M30 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m30 mcp server export"
```
