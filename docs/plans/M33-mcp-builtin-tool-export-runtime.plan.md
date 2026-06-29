# M33 MCP Built-in Tool Export Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let MCP stdio export profiles expose curated built-in workspace inspection tools through explicit manifests and a workspace sandbox.

**Architecture:** `agent-mcp` owns export profile and catalog contracts. `agent-tool` owns built-in tool behavior. `agent-cli` owns args, path IO, and stdio wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M33 Technical Design

**Files:**
- Create: `docs/specs/M33-00-mcp-builtin-tool-export-technical-design.spec.md`
- Create: `docs/plans/M33-00-mcp-builtin-tool-export-technical-design.plan.md`

- [x] **Step 1: Write the M33 technical design baseline**

Define the built-in export allow-list, workspace root requirement, manifest tool binding, CLI flags, policy path, privacy rules, and verification gates.

### Task 2: Decompose M33 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M33-01-builtin-tool-export-catalog.spec.md`
- Create: `docs/plans/M33-01-builtin-tool-export-catalog.plan.md`
- Create: `docs/specs/M33-02-manifest-tool-binding-diagnostics.spec.md`
- Create: `docs/plans/M33-02-manifest-tool-binding-diagnostics.plan.md`
- Create: `docs/specs/M33-03-cli-workspace-tool-profile.spec.md`
- Create: `docs/plans/M33-03-cli-workspace-tool-profile.plan.md`
- Create: `docs/specs/M33-04-stdio-tool-call-e2e.spec.md`
- Create: `docs/plans/M33-04-stdio-tool-call-e2e.plan.md`
- Create: `docs/specs/M33-05-hardening-docs.spec.md`
- Create: `docs/plans/M33-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M33 Slices in Order

**Files:**
- Execute: `docs/plans/M33-01-builtin-tool-export-catalog.plan.md`
- Execute: `docs/plans/M33-02-manifest-tool-binding-diagnostics.plan.md`
- Execute: `docs/plans/M33-03-cli-workspace-tool-profile.plan.md`
- Execute: `docs/plans/M33-04-stdio-tool-call-e2e.plan.md`
- Execute: `docs/plans/M33-05-hardening-docs.plan.md`

- [ ] **Step 1: Add built-in tool export catalog**
- [ ] **Step 2: Harden manifest tool binding diagnostics**
- [ ] **Step 3: Add CLI workspace-backed profile startup**
- [ ] **Step 4: Prove stdio tool list and call flow**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M33

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M33-mcp-builtin-tool-export-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m33 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M33 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m33 mcp builtin tool export runtime"
```
