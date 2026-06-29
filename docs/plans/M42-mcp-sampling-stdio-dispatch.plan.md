# M42 MCP Sampling Stdio Dispatch Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add fixture-grade stdio JSON-RPC dispatch for server-initiated MCP `sampling/createMessage` requests.

**Architecture:** `agent-mcp` owns JSON-RPC frame parsing, validation, provider invocation through `McpSamplingProvider`, response rendering, and redacted diagnostics. `agent-core` remains the model-backed provider owner from M41.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M42 Technical Design

**Files:**
- Create: `docs/specs/M42-00-mcp-sampling-stdio-dispatch-technical-design.spec.md`
- Create: `docs/plans/M42-00-mcp-sampling-stdio-dispatch-technical-design.plan.md`

- [x] **Step 1: Write the M42 technical design baseline**

Define crate ownership, JSON-RPC frame shape, dispatcher flow, validation-before-provider behavior, error redaction, and verification gates.

### Task 2: Decompose M42 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M42-01-sampling-stdio-frame-contracts.spec.md`
- Create: `docs/plans/M42-01-sampling-stdio-frame-contracts.plan.md`
- Create: `docs/specs/M42-02-sampling-stdio-dispatcher.spec.md`
- Create: `docs/plans/M42-02-sampling-stdio-dispatcher.plan.md`
- Create: `docs/specs/M42-03-sampling-stdio-validation.spec.md`
- Create: `docs/plans/M42-03-sampling-stdio-validation.plan.md`
- Create: `docs/specs/M42-04-sampling-stdio-redaction.spec.md`
- Create: `docs/plans/M42-04-sampling-stdio-redaction.plan.md`
- Create: `docs/specs/M42-05-hardening-docs.spec.md`
- Create: `docs/plans/M42-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M42 Slices in Order

**Files:**
- Execute: `docs/plans/M42-01-sampling-stdio-frame-contracts.plan.md`
- Execute: `docs/plans/M42-02-sampling-stdio-dispatcher.plan.md`
- Execute: `docs/plans/M42-03-sampling-stdio-validation.plan.md`
- Execute: `docs/plans/M42-04-sampling-stdio-redaction.plan.md`
- Execute: `docs/plans/M42-05-hardening-docs.plan.md`

- [x] **Step 1: Add sampling stdio frame contracts**
- [x] **Step 2: Add provider-backed sampling stdio dispatcher**
- [x] **Step 3: Add validation-before-provider guardrails**
- [x] **Step 4: Add redacted public diagnostics**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M42

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M42-mcp-sampling-stdio-dispatch.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M42 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m42 mcp sampling stdio dispatch"
```
