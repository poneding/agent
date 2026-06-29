# M34 MCP Export Profile Inspection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed MCP export profile inspection and CLI preflight output before stdio startup.

**Architecture:** `agent-mcp` owns inspection contracts and diagnostics. `agent-cli` owns manifest file IO, args, and rendering.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M34 Technical Design

**Files:**
- Create: `docs/specs/M34-00-mcp-export-profile-inspection-technical-design.spec.md`
- Create: `docs/plans/M34-00-mcp-export-profile-inspection-technical-design.plan.md`

- [x] **Step 1: Write the M34 technical design baseline**

Define inspection ownership, diagnostics, CLI command shape, JSON contracts, redaction, and verification gates.

### Task 2: Decompose M34 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M34-01-profile-inspection-contracts.spec.md`
- Create: `docs/plans/M34-01-profile-inspection-contracts.plan.md`
- Create: `docs/specs/M34-02-cli-profile-inspect-command.spec.md`
- Create: `docs/plans/M34-02-cli-profile-inspect-command.plan.md`
- Create: `docs/specs/M34-03-preflight-diagnostics.spec.md`
- Create: `docs/plans/M34-03-preflight-diagnostics.plan.md`
- Create: `docs/specs/M34-04-json-redaction-regressions.spec.md`
- Create: `docs/plans/M34-04-json-redaction-regressions.plan.md`
- Create: `docs/specs/M34-05-hardening-docs.spec.md`
- Create: `docs/plans/M34-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M34 Slices in Order

**Files:**
- Execute: `docs/plans/M34-01-profile-inspection-contracts.plan.md`
- Execute: `docs/plans/M34-02-cli-profile-inspect-command.plan.md`
- Execute: `docs/plans/M34-03-preflight-diagnostics.plan.md`
- Execute: `docs/plans/M34-04-json-redaction-regressions.plan.md`
- Execute: `docs/plans/M34-05-hardening-docs.plan.md`

- [x] **Step 1: Add profile inspection contracts**
- [x] **Step 2: Add CLI profile inspect command**
- [x] **Step 3: Add preflight diagnostics**
- [x] **Step 4: Add JSON redaction regressions**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M34

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M34-mcp-export-profile-inspection.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m34 cargo xtask ci
```

Expected: all checks pass.

Verified:

```text
Quality gate ci-local status=passed
total=4 passed=4 failed=0 skipped=0
fmt-check status=passed
clippy status=passed
test status=passed
eval status=passed
```

- [x] **Step 2: Mark M34 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m34 mcp export profile inspection"
```
