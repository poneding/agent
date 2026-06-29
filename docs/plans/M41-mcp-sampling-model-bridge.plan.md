# M41 MCP Sampling Model Bridge Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe text-only bridge from validated MCP sampling requests to `agent-model` providers.

**Architecture:** `agent-mcp` keeps protocol contracts and validation. `agent-core` adds an `mcp_sampling` module that maps MCP sampling requests into `agent-model::ModelRequest` and implements a provider-backed `McpSamplingProvider`.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M41 Technical Design

**Files:**
- Create: `docs/specs/M41-00-mcp-sampling-model-bridge-technical-design.spec.md`
- Create: `docs/plans/M41-00-mcp-sampling-model-bridge-technical-design.plan.md`

- [x] **Step 1: Write the M41 technical design baseline**

Define crate ownership, bridge contracts, text-only mapping, unsupported content handling, provider execution, redaction, and verification gates.

### Task 2: Decompose M41 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M41-01-sampling-model-request-mapping.spec.md`
- Create: `docs/plans/M41-01-sampling-model-request-mapping.plan.md`
- Create: `docs/specs/M41-02-sampling-model-provider.spec.md`
- Create: `docs/plans/M41-02-sampling-model-provider.plan.md`
- Create: `docs/specs/M41-03-sampling-bridge-redaction.spec.md`
- Create: `docs/plans/M41-03-sampling-bridge-redaction.plan.md`
- Create: `docs/specs/M41-04-sampling-bridge-guardrails.spec.md`
- Create: `docs/plans/M41-04-sampling-bridge-guardrails.plan.md`
- Create: `docs/specs/M41-05-hardening-docs.spec.md`
- Create: `docs/plans/M41-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M41 Slices in Order

**Files:**
- Execute: `docs/plans/M41-01-sampling-model-request-mapping.plan.md`
- Execute: `docs/plans/M41-02-sampling-model-provider.plan.md`
- Execute: `docs/plans/M41-03-sampling-bridge-redaction.plan.md`
- Execute: `docs/plans/M41-04-sampling-bridge-guardrails.plan.md`
- Execute: `docs/plans/M41-05-hardening-docs.plan.md`

- [x] **Step 1: Add sampling model request mapping**
- [x] **Step 2: Add provider-backed sampling execution**
- [x] **Step 3: Add redacted bridge diagnostics**
- [x] **Step 4: Add unsupported-path guardrails**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M41

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M41-mcp-sampling-model-bridge.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M41 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m41 mcp sampling model bridge"
```
