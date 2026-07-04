# M40 MCP Sampling Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe MCP sampling runtime foundation for `sampling/createMessage` client requests.

**Architecture:** `agent-mcp` owns sampling protocol contracts, safe-subset validation, fixture provider behavior, and gateway diagnostics. Later milestones can bridge approved requests into `agent-model` and UI approval surfaces.

**Tech Stack:** Rust 2024 edition, serde JSON, async-trait, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M40 Technical Design

**Files:**
- Create: `docs/specs/M40-00-mcp-sampling-technical-design.spec.md`
- Create: `docs/plans/M40-00-mcp-sampling-technical-design.plan.md`

- [x] **Step 1: Write the M40 technical design baseline**

Define sampling directionality, capability negotiation, request/result contracts, safe subset, validation policy, fixture behavior, gateway diagnostics, privacy, and verification gates.

### Task 2: Decompose M40 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M40-01-sampling-contracts.spec.md`
- Create: `docs/plans/M40-01-sampling-contracts.plan.md`
- Create: `docs/specs/M40-02-sampling-validation.spec.md`
- Create: `docs/plans/M40-02-sampling-validation.plan.md`
- Create: `docs/specs/M40-03-client-session-sampling.spec.md`
- Create: `docs/plans/M40-03-client-session-sampling.plan.md`
- Create: `docs/specs/M40-04-gateway-sampling-diagnostics.spec.md`
- Create: `docs/plans/M40-04-gateway-sampling-diagnostics.plan.md`
- Create: `docs/specs/M40-05-hardening-docs.spec.md`
- Create: `docs/plans/M40-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M40 Slices in Order

**Files:**
- Execute: `docs/plans/M40-01-sampling-contracts.plan.md`
- Execute: `docs/plans/M40-02-sampling-validation.plan.md`
- Execute: `docs/plans/M40-03-client-session-sampling.plan.md`
- Execute: `docs/plans/M40-04-gateway-sampling-diagnostics.plan.md`
- Execute: `docs/plans/M40-05-hardening-docs.plan.md`

- [x] **Step 1: Add sampling contracts**
- [x] **Step 2: Add safe sampling request validation**
- [x] **Step 3: Add client session sampling fixture behavior**
- [x] **Step 4: Wire gateway sampling diagnostics**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M40

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M40-mcp-sampling-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Verified:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo xtask ci
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo xtask ci
```

Expected: all checks pass.

- [x] **Step 2: Mark M40 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m40 mcp sampling runtime"
```
