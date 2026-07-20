# M53 MCP HTTP OAuth Authorization Flow Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed, redacted OAuth authorization-flow contracts for MCP HTTP clients.

**Architecture:** `ruse-mcp` owns pure flow planning and validation contracts; external UI/auth/storage layers own browser launch, callback servers, remote token exchange execution, and durable secret storage.

**Tech Stack:** Rust 2024 edition, serde, SHA-256/base64url helpers as needed, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M53 Technical Design

**Files:**
- Create: `docs/specs/M53-00-mcp-http-oauth-authorization-flow-technical-design.spec.md`
- Create: `docs/plans/M53-00-mcp-http-oauth-authorization-flow-technical-design.plan.md`

- [x] **Step 1: Write the M53 technical design baseline**

Define crate ownership, PKCE/state boundaries, authorization URL planning, callback validation, token exchange DTO scope, redaction behavior, and verification gates.

### Task 2: Decompose M53 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M53-01-pkce-state-contracts.spec.md`
- Create: `docs/plans/M53-01-pkce-state-contracts.plan.md`
- Create: `docs/specs/M53-02-authorization-request-planning.spec.md`
- Create: `docs/plans/M53-02-authorization-request-planning.plan.md`
- Create: `docs/specs/M53-03-callback-token-exchange-contracts.spec.md`
- Create: `docs/plans/M53-03-callback-token-exchange-contracts.plan.md`
- Create: `docs/specs/M53-04-flow-diagnostics-redaction.spec.md`
- Create: `docs/plans/M53-04-flow-diagnostics-redaction.plan.md`
- Create: `docs/specs/M53-05-hardening-docs.spec.md`
- Create: `docs/plans/M53-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M53 Slices in Order

**Files:**
- Execute: `docs/plans/M53-01-pkce-state-contracts.plan.md`
- Execute: `docs/plans/M53-02-authorization-request-planning.plan.md`
- Execute: `docs/plans/M53-03-callback-token-exchange-contracts.plan.md`
- Execute: `docs/plans/M53-04-flow-diagnostics-redaction.plan.md`
- Execute: `docs/plans/M53-05-hardening-docs.plan.md`

- [x] **Step 1: Add PKCE and state contracts**
- [x] **Step 2: Add authorization request planning**
- [x] **Step 3: Add callback validation and token exchange DTOs**
- [x] **Step 4: Add flow diagnostics and redaction hardening**
- [x] **Step 5: Add docs and final verification**

### Task 4: Complete M53

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M53-mcp-http-oauth-authorization-flow-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo xtask ci
```

Expected: all checks pass.

Evidence, 2026-06-30:

```bash
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_pkce_state
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_authorization_request
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_callback
CARGO_TARGET_DIR=target/codex-m53 cargo test -p ruse-mcp mcp_client_http_oauth_flow_diagnostics
git diff --check
CARGO_TARGET_DIR=target/codex-m53 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m53 cargo xtask ci
```

- [x] **Step 2: Mark M53 complete and commit**

Run:

```bash
git add README.md docs crates Cargo.toml Cargo.lock
git commit -m "feat: complete m53 mcp http oauth authorization flow runtime"
```
