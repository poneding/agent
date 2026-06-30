# M51 MCP HTTP OAuth Authorization Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe OAuth authorization foundation for MCP streamable HTTP clients without owning browser login, token storage, or provider-specific auth flows.

**Architecture:** `agent-mcp` owns OAuth metadata contracts, diagnostics, and explicit bearer-token request construction. Secret/auth layers supply token material later.

**Tech Stack:** Rust 2024 edition, serde, fixture transports, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M51 Technical Design

**Files:**
- Create: `docs/specs/M51-00-mcp-http-oauth-authorization-technical-design.spec.md`
- Create: `docs/plans/M51-00-mcp-http-oauth-authorization-technical-design.plan.md`

- [x] **Step 1: Write the M51 technical design baseline**

Define crate ownership, metadata contracts, explicit token boundary, redaction behavior, diagnostics, and verification gates.

### Task 2: Decompose M51 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M51-01-oauth-metadata-contracts.spec.md`
- Create: `docs/plans/M51-01-oauth-metadata-contracts.plan.md`
- Create: `docs/specs/M51-02-protected-resource-discovery.spec.md`
- Create: `docs/plans/M51-02-protected-resource-discovery.plan.md`
- Create: `docs/specs/M51-03-authorization-policy-diagnostics.spec.md`
- Create: `docs/plans/M51-03-authorization-policy-diagnostics.plan.md`
- Create: `docs/specs/M51-04-bearer-token-request-boundary.spec.md`
- Create: `docs/plans/M51-04-bearer-token-request-boundary.plan.md`
- Create: `docs/specs/M51-05-hardening-docs.spec.md`
- Create: `docs/plans/M51-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M51 Slices in Order

**Files:**
- Execute: `docs/plans/M51-01-oauth-metadata-contracts.plan.md`
- Execute: `docs/plans/M51-02-protected-resource-discovery.plan.md`
- Execute: `docs/plans/M51-03-authorization-policy-diagnostics.plan.md`
- Execute: `docs/plans/M51-04-bearer-token-request-boundary.plan.md`
- Execute: `docs/plans/M51-05-hardening-docs.plan.md`

- [x] **Step 1: Add OAuth metadata contracts**
- [x] **Step 2: Add protected-resource discovery contracts**
- [x] **Step 3: Add authorization diagnostics**
- [x] **Step 4: Add bearer-token request boundary**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M51

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M51-mcp-http-oauth-authorization-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m51 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M51 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m51 mcp http oauth authorization runtime"
```
