# M39 MCP Client Roots Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe MCP client roots support for external MCP servers connected through `agent` gateway flows.

**Architecture:** `agent-mcp` owns root contracts, registry validation, fixture client behavior, and gateway diagnostics. CLI/Desktop remain consumers of typed runtime state.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M39 Technical Design

**Files:**
- Create: `docs/specs/M39-00-mcp-client-roots-technical-design.spec.md`
- Create: `docs/plans/M39-00-mcp-client-roots-technical-design.plan.md`

- [x] **Step 1: Write the M39 technical design baseline**

Define MCP roots directionality, root contracts, registry validation, client capability metadata, gateway diagnostics, privacy, and verification gates.

### Task 2: Decompose M39 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M39-01-client-root-contracts.spec.md`
- Create: `docs/plans/M39-01-client-root-contracts.plan.md`
- Create: `docs/specs/M39-02-client-root-registry.spec.md`
- Create: `docs/plans/M39-02-client-root-registry.plan.md`
- Create: `docs/specs/M39-03-client-session-roots.spec.md`
- Create: `docs/plans/M39-03-client-session-roots.plan.md`
- Create: `docs/specs/M39-04-gateway-roots-integration.spec.md`
- Create: `docs/plans/M39-04-gateway-roots-integration.plan.md`
- Create: `docs/specs/M39-05-hardening-docs.spec.md`
- Create: `docs/plans/M39-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M39 Slices in Order

**Files:**
- Execute: `docs/plans/M39-01-client-root-contracts.plan.md`
- Execute: `docs/plans/M39-02-client-root-registry.plan.md`
- Execute: `docs/plans/M39-03-client-session-roots.plan.md`
- Execute: `docs/plans/M39-04-gateway-roots-integration.plan.md`
- Execute: `docs/plans/M39-05-hardening-docs.plan.md`

- [x] **Step 1: Add client root contracts**
- [x] **Step 2: Add root registry validation**
- [ ] **Step 3: Add client session roots behavior**
- [ ] **Step 4: Wire gateway roots diagnostics**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M39

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M39-mcp-client-roots-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m39 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M39 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m39 mcp client roots runtime"
```
