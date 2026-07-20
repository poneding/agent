# M45 MCP Client Streamable HTTP Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real client-side MCP streamable HTTP transport path for initialize, tools/list, and tools/call through the existing typed gateway boundary.

**Architecture:** `ruse-mcp` owns HTTP JSON-RPC body contracts, transport abstraction, typed result mapping, gateway client integration, redaction, and tests. CLI/Desktop remain consumers of typed runtime contracts.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, HTTP transport abstraction, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M45 Technical Design

**Files:**
- Create: `docs/specs/M45-00-mcp-client-streamable-http-technical-design.spec.md`
- Create: `docs/plans/M45-00-mcp-client-streamable-http-technical-design.plan.md`

- [x] **Step 1: Write the M45 technical design baseline**

Define crate ownership, HTTP request/response contracts, transport trait boundaries, session state, timeout/redaction behavior, and verification gates.

### Task 2: Decompose M45 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M45-01-http-client-contracts.spec.md`
- Create: `docs/plans/M45-01-http-client-contracts.plan.md`
- Create: `docs/specs/M45-02-http-transport-runtime.spec.md`
- Create: `docs/plans/M45-02-http-transport-runtime.plan.md`
- Create: `docs/specs/M45-03-http-mcp-client-discovery.spec.md`
- Create: `docs/plans/M45-03-http-mcp-client-discovery.plan.md`
- Create: `docs/specs/M45-04-http-gateway-tool-call.spec.md`
- Create: `docs/plans/M45-04-http-gateway-tool-call.plan.md`
- Create: `docs/specs/M45-05-hardening-docs.spec.md`
- Create: `docs/plans/M45-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M45 Slices in Order

**Files:**
- Execute: `docs/plans/M45-01-http-client-contracts.plan.md`
- Execute: `docs/plans/M45-02-http-transport-runtime.plan.md`
- Execute: `docs/plans/M45-03-http-mcp-client-discovery.plan.md`
- Execute: `docs/plans/M45-04-http-gateway-tool-call.plan.md`
- Execute: `docs/plans/M45-05-hardening-docs.plan.md`

- [x] **Step 1: Add HTTP JSON-RPC client contracts**
- [x] **Step 2: Add HTTP transport runtime**
- [x] **Step 3: Implement `McpClient` over HTTP for initialize and tools/list**
- [x] **Step 4: Add gateway HTTP tools/call execution**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M45

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M45-mcp-client-streamable-http.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m45 cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark M45 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m45 mcp client streamable http"
```

## Verification Evidence

2026-06-30:

- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_contracts` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_transport` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_client_http_mcp_client` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo test -p ruse-mcp mcp_gateway_http_tool_call` passed.
- `git diff --check` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo fmt --all -- --check` passed.
- `CARGO_TARGET_DIR=target/codex-m45 cargo xtask ci` passed.
