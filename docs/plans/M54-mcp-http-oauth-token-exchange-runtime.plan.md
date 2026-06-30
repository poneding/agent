# M54 MCP HTTP OAuth Token Exchange Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, redacted MCP HTTP OAuth token exchange execution for authorization-code flows.

**Architecture:** `agent-mcp` owns exchange request execution, response parsing, diagnostics, and lease conversion. UI/auth/vault layers own browser launch, callback hosting, account UX, refresh/revocation/DCR execution, and durable secret storage.

**Tech Stack:** Rust 2024 edition, serde, async-trait, reqwest form POST boundary, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M54 Technical Design

**Files:**
- Create: `docs/specs/M54-00-mcp-http-oauth-token-exchange-technical-design.spec.md`
- Create: `docs/plans/M54-00-mcp-http-oauth-token-exchange-technical-design.plan.md`

- [x] **Step 1: Write the M54 technical design baseline**

Define crate ownership, exchange transport boundary, token response parsing, redaction behavior, lease conversion, and verification gates.

### Task 2: Decompose M54 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M54-01-token-response-contracts.spec.md`
- Create: `docs/plans/M54-01-token-response-contracts.plan.md`
- Create: `docs/specs/M54-02-token-exchange-transport.spec.md`
- Create: `docs/plans/M54-02-token-exchange-transport.plan.md`
- Create: `docs/specs/M54-03-token-exchange-runner-diagnostics.spec.md`
- Create: `docs/plans/M54-03-token-exchange-runner-diagnostics.plan.md`
- Create: `docs/specs/M54-04-lease-conversion-redaction.spec.md`
- Create: `docs/plans/M54-04-lease-conversion-redaction.plan.md`
- Create: `docs/specs/M54-05-hardening-docs.spec.md`
- Create: `docs/plans/M54-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M54 Slices in Order

**Files:**
- Execute: `docs/plans/M54-01-token-response-contracts.plan.md`
- Execute: `docs/plans/M54-02-token-exchange-transport.plan.md`
- Execute: `docs/plans/M54-03-token-exchange-runner-diagnostics.plan.md`
- Execute: `docs/plans/M54-04-lease-conversion-redaction.plan.md`
- Execute: `docs/plans/M54-05-hardening-docs.plan.md`

- [x] **Step 1: Add token response contracts**
- [x] **Step 2: Add token exchange transport boundary**
- [x] **Step 3: Add exchange runner diagnostics**
- [x] **Step 4: Add lease conversion and redaction hardening**
- [x] **Step 5: Add docs and final verification**

### Task 4: Complete M54

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M54-mcp-http-oauth-token-exchange-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m54 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M54 complete and commit**

Run:

```bash
git add README.md docs crates Cargo.toml Cargo.lock
git commit -m "feat: complete m54 mcp http oauth token exchange runtime"
```
