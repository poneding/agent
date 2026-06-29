# M36 MCP Prompt Arguments Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit, safe string arguments for MCP prompt exports.

**Architecture:** `agent-mcp` owns prompt argument contracts, validation, and rendering. `agent-cli` owns manifest file IO and stdio startup wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M36 Technical Design

**Files:**
- Create: `docs/specs/M36-00-mcp-prompt-arguments-technical-design.spec.md`
- Create: `docs/plans/M36-00-mcp-prompt-arguments-technical-design.plan.md`

- [ ] **Step 1: Write the M36 technical design baseline**

Define prompt argument ownership, string-only scope, template rendering, manifest shape, CLI boundaries, redaction, and verification gates.

### Task 2: Decompose M36 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M36-01-prompt-argument-contracts.spec.md`
- Create: `docs/plans/M36-01-prompt-argument-contracts.plan.md`
- Create: `docs/specs/M36-02-prompt-template-rendering.spec.md`
- Create: `docs/plans/M36-02-prompt-template-rendering.plan.md`
- Create: `docs/specs/M36-03-manifest-prompt-arguments.spec.md`
- Create: `docs/plans/M36-03-manifest-prompt-arguments.plan.md`
- Create: `docs/specs/M36-04-cli-stdio-prompt-arguments.spec.md`
- Create: `docs/plans/M36-04-cli-stdio-prompt-arguments.plan.md`
- Create: `docs/specs/M36-05-hardening-docs.spec.md`
- Create: `docs/plans/M36-05-hardening-docs.plan.md`

- [ ] **Step 1: Create slice specs**
- [ ] **Step 2: Create slice plans**

### Task 3: Implement M36 Slices in Order

**Files:**
- Execute: `docs/plans/M36-01-prompt-argument-contracts.plan.md`
- Execute: `docs/plans/M36-02-prompt-template-rendering.plan.md`
- Execute: `docs/plans/M36-03-manifest-prompt-arguments.plan.md`
- Execute: `docs/plans/M36-04-cli-stdio-prompt-arguments.plan.md`
- Execute: `docs/plans/M36-05-hardening-docs.plan.md`

- [x] **Step 1: Add prompt argument contracts**
- [x] **Step 2: Add template rendering and argument validation**
- [ ] **Step 3: Add manifest prompt argument declarations**
- [ ] **Step 4: Wire CLI stdio prompt arguments**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M36

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M36-mcp-prompt-arguments-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m36 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M36 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m36 mcp prompt arguments runtime"
```
