# M35 MCP Prompt Export Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add MCP server prompt export support backed by explicit prompt profiles and manifest declarations.

**Architecture:** `agent-mcp` owns protocol contracts and prompt registry behavior. `agent-cli` owns manifest file IO and stdio startup wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M35 Technical Design

**Files:**
- Create: `docs/specs/M35-00-mcp-prompt-export-technical-design.spec.md`
- Create: `docs/plans/M35-00-mcp-prompt-export-technical-design.plan.md`

- [x] **Step 1: Write the M35 technical design baseline**

Define prompt export ownership, MCP method support, manifest shape, CLI boundaries, redaction, and verification gates.

### Task 2: Decompose M35 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M35-01-prompt-export-contracts.spec.md`
- Create: `docs/plans/M35-01-prompt-export-contracts.plan.md`
- Create: `docs/specs/M35-02-prompt-registry-and-protocol.spec.md`
- Create: `docs/plans/M35-02-prompt-registry-and-protocol.plan.md`
- Create: `docs/specs/M35-03-manifest-prompt-profile.spec.md`
- Create: `docs/plans/M35-03-manifest-prompt-profile.plan.md`
- Create: `docs/specs/M35-04-cli-stdio-prompt-profile.spec.md`
- Create: `docs/plans/M35-04-cli-stdio-prompt-profile.plan.md`
- Create: `docs/specs/M35-05-hardening-docs.spec.md`
- Create: `docs/plans/M35-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M35 Slices in Order

**Files:**
- Execute: `docs/plans/M35-01-prompt-export-contracts.plan.md`
- Execute: `docs/plans/M35-02-prompt-registry-and-protocol.plan.md`
- Execute: `docs/plans/M35-03-manifest-prompt-profile.plan.md`
- Execute: `docs/plans/M35-04-cli-stdio-prompt-profile.plan.md`
- Execute: `docs/plans/M35-05-hardening-docs.plan.md`

- [x] **Step 1: Add prompt export contracts**
- [x] **Step 2: Add prompt registry and MCP protocol methods**
- [x] **Step 3: Add manifest prompt declarations**
- [ ] **Step 4: Wire CLI stdio prompt profiles**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M35

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M35-mcp-prompt-export-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m35 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M35 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m35 mcp prompt export runtime"
```
