# M38 MCP Completion Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe MCP `completion/complete` support for prompt arguments and resource template variables.

**Architecture:** `agent-mcp` owns completion contracts, completion source lookup, validation, redaction, and JSON-RPC behavior. `agent-cli` owns manifest file IO and stdio startup wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M38 Technical Design

**Files:**
- Create: `docs/specs/M38-00-mcp-completion-technical-design.spec.md`
- Create: `docs/plans/M38-00-mcp-completion-technical-design.plan.md`

- [x] **Step 1: Write the M38 technical design baseline**

Define completion request shape, ref handling, source ownership, manifest shape, bounds, redaction, and verification gates.

### Task 2: Decompose M38 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M38-01-completion-contracts.spec.md`
- Create: `docs/plans/M38-01-completion-contracts.plan.md`
- Create: `docs/specs/M38-02-prompt-argument-completions.spec.md`
- Create: `docs/plans/M38-02-prompt-argument-completions.plan.md`
- Create: `docs/specs/M38-03-resource-template-completions.spec.md`
- Create: `docs/plans/M38-03-resource-template-completions.plan.md`
- Create: `docs/specs/M38-04-cli-stdio-completions.spec.md`
- Create: `docs/plans/M38-04-cli-stdio-completions.plan.md`
- Create: `docs/specs/M38-05-hardening-docs.spec.md`
- Create: `docs/plans/M38-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M38 Slices in Order

**Files:**
- Execute: `docs/plans/M38-01-completion-contracts.plan.md`
- Execute: `docs/plans/M38-02-prompt-argument-completions.plan.md`
- Execute: `docs/plans/M38-03-resource-template-completions.plan.md`
- Execute: `docs/plans/M38-04-cli-stdio-completions.plan.md`
- Execute: `docs/plans/M38-05-hardening-docs.plan.md`

- [x] **Step 1: Add completion contracts**
- [ ] **Step 2: Add prompt argument completion lookup**
- [ ] **Step 3: Add resource template variable completion lookup**
- [ ] **Step 4: Wire CLI stdio completions**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M38

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M38-mcp-completion-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m38 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M38 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m38 mcp completion runtime"
```
