# M32 MCP Export Profile Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Start the MCP stdio server from explicit, validated export profiles.

**Architecture:** `agent-mcp` owns manifest contracts and profile construction. `agent-cli` owns argument parsing, file IO, and stdio process wiring.

**Tech Stack:** Rust 2024 edition, serde JSON, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M32 Technical Design

**Files:**
- Create: `docs/specs/M32-00-mcp-export-profile-technical-design.spec.md`
- Create: `docs/plans/M32-00-mcp-export-profile-technical-design.plan.md`

- [x] **Step 1: Write the M32 technical design baseline**

Define manifest ownership, profile construction, file resource boundaries, built-in tool export boundaries, CLI flags, privacy rules, and verification gates.

### Task 2: Decompose M32 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M32-01-export-manifest-contracts.spec.md`
- Create: `docs/plans/M32-01-export-manifest-contracts.plan.md`
- Create: `docs/specs/M32-02-resource-export-loader.spec.md`
- Create: `docs/plans/M32-02-resource-export-loader.plan.md`
- Create: `docs/specs/M32-03-tool-export-catalog.spec.md`
- Create: `docs/plans/M32-03-tool-export-catalog.plan.md`
- Create: `docs/specs/M32-04-cli-profile-stdio.spec.md`
- Create: `docs/plans/M32-04-cli-profile-stdio.plan.md`
- Create: `docs/specs/M32-05-runtime-hardening-docs.spec.md`
- Create: `docs/plans/M32-05-runtime-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M32 Slices in Order

**Files:**
- Execute: `docs/plans/M32-01-export-manifest-contracts.plan.md`
- Execute: `docs/plans/M32-02-resource-export-loader.plan.md`
- Execute: `docs/plans/M32-03-tool-export-catalog.plan.md`
- Execute: `docs/plans/M32-04-cli-profile-stdio.plan.md`
- Execute: `docs/plans/M32-05-runtime-hardening-docs.plan.md`

- [x] **Step 1: Add export manifest contracts**
- [x] **Step 2: Add resource export loader**
- [x] **Step 3: Add tool export catalog**
- [x] **Step 4: Add CLI profile stdio flags**
- [x] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M32

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M32-mcp-export-profile-runtime.plan.md`

- [x] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask ci
```

Expected: all checks pass.

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m32 cargo xtask ci
```

Quality gate summary:

```text
Quality gate ci-local status=passed
total=4 passed=4 failed=0 skipped=0
fmt-check status=passed
clippy status=passed
test status=passed
eval status=passed
```

- [x] **Step 2: Mark M32 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m32 mcp export profile runtime"
```
