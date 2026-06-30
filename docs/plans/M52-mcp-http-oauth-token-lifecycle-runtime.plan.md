# M52 MCP HTTP OAuth Token Lifecycle Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe token lifecycle contracts, diagnostics, refresh planning, and request factories for MCP HTTP OAuth authorization.

**Architecture:** `agent-mcp` owns typed, pure token lifecycle contracts and request factory helpers; external auth/storage layers own login, refresh network calls, and durable secret storage.

**Tech Stack:** Rust 2024 edition, serde, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M52 Technical Design

**Files:**
- Create: `docs/specs/M52-00-mcp-http-oauth-token-lifecycle-technical-design.spec.md`
- Create: `docs/plans/M52-00-mcp-http-oauth-token-lifecycle-technical-design.plan.md`

- [x] **Step 1: Write the M52 technical design baseline**

Define crate ownership, token lease boundaries, redaction behavior, refresh planning, request-factory scope, and verification gates.

### Task 2: Decompose M52 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M52-01-oauth-token-lease-contracts.spec.md`
- Create: `docs/plans/M52-01-oauth-token-lease-contracts.plan.md`
- Create: `docs/specs/M52-02-token-source-reference-boundary.spec.md`
- Create: `docs/plans/M52-02-token-source-reference-boundary.plan.md`
- Create: `docs/specs/M52-03-refresh-planning-diagnostics.spec.md`
- Create: `docs/plans/M52-03-refresh-planning-diagnostics.plan.md`
- Create: `docs/specs/M52-04-authorized-request-factory.spec.md`
- Create: `docs/plans/M52-04-authorized-request-factory.plan.md`
- Create: `docs/specs/M52-05-hardening-docs.spec.md`
- Create: `docs/plans/M52-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M52 Slices in Order

**Files:**
- Execute: `docs/plans/M52-01-oauth-token-lease-contracts.plan.md`
- Execute: `docs/plans/M52-02-token-source-reference-boundary.plan.md`
- Execute: `docs/plans/M52-03-refresh-planning-diagnostics.plan.md`
- Execute: `docs/plans/M52-04-authorized-request-factory.plan.md`
- Execute: `docs/plans/M52-05-hardening-docs.plan.md`

- [ ] **Step 1: Add OAuth token lease contracts**
- [ ] **Step 2: Add token source reference boundary**
- [ ] **Step 3: Add refresh planning diagnostics**
- [ ] **Step 4: Add authorized request factory**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M52

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M52-mcp-http-oauth-token-lifecycle-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m52 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M52 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m52 mcp http oauth token lifecycle runtime"
```
