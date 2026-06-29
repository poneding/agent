# M37 MCP Resource Templates Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit, safe MCP resource templates for parameterized resource reads.

**Architecture:** `agent-mcp` owns resource template contracts, matching, validation, policy, and JSON-RPC behavior. `agent-cli` owns manifest file IO and stdio startup wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M37 Technical Design

**Files:**
- Create: `docs/specs/M37-00-mcp-resource-templates-technical-design.spec.md`
- Create: `docs/plans/M37-00-mcp-resource-templates-technical-design.plan.md`

- [x] **Step 1: Write the M37 technical design baseline**

Define resource template ownership, URI template syntax, path resolution, manifest shape, CLI boundaries, redaction, and verification gates.

### Task 2: Decompose M37 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M37-01-resource-template-contracts.spec.md`
- Create: `docs/plans/M37-01-resource-template-contracts.plan.md`
- Create: `docs/specs/M37-02-resource-template-matching.spec.md`
- Create: `docs/plans/M37-02-resource-template-matching.plan.md`
- Create: `docs/specs/M37-03-manifest-resource-templates.spec.md`
- Create: `docs/plans/M37-03-manifest-resource-templates.plan.md`
- Create: `docs/specs/M37-04-cli-stdio-resource-templates.spec.md`
- Create: `docs/plans/M37-04-cli-stdio-resource-templates.plan.md`
- Create: `docs/specs/M37-05-hardening-docs.spec.md`
- Create: `docs/plans/M37-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M37 Slices in Order

**Files:**
- Execute: `docs/plans/M37-01-resource-template-contracts.plan.md`
- Execute: `docs/plans/M37-02-resource-template-matching.plan.md`
- Execute: `docs/plans/M37-03-manifest-resource-templates.plan.md`
- Execute: `docs/plans/M37-04-cli-stdio-resource-templates.plan.md`
- Execute: `docs/plans/M37-05-hardening-docs.plan.md`

- [x] **Step 1: Add resource template contracts**
- [x] **Step 2: Add concrete URI matching and path resolution**
- [x] **Step 3: Add manifest resource template declarations**
- [x] **Step 4: Wire CLI stdio resource templates**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M37

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M37-mcp-resource-templates-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo xtask ci
```

Expected: all checks pass.

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m37 cargo xtask ci
```

- [x] **Step 2: Mark M37 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m37 mcp resource templates runtime"
```
