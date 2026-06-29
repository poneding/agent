# M31 MCP Stdio Server Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote M30 MCP server fixtures into a runnable stdio server runtime and CLI command.

**Architecture:** `agent-mcp` owns session, export profile, and trace contracts. `agent-cli` owns stdin/stdout process wiring only.

**Tech Stack:** Rust 2024 edition, serde JSON, stdio IO traits, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M31 Technical Design

**Files:**
- Create: `docs/specs/M31-00-mcp-stdio-server-technical-design.spec.md`
- Create: `docs/plans/M31-00-mcp-stdio-server-technical-design.plan.md`

- [x] **Step 1: Write the M31 technical design baseline**

Define session ownership, stdio loop boundaries, profile bootstrap, trace/audit output, privacy rules, and verification gates.

### Task 2: Decompose M31 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M31-01-stdio-session-runtime.spec.md`
- Create: `docs/plans/M31-01-stdio-session-runtime.plan.md`
- Create: `docs/specs/M31-02-export-profile-bootstrap.spec.md`
- Create: `docs/plans/M31-02-export-profile-bootstrap.plan.md`
- Create: `docs/specs/M31-03-cli-stdio-command.spec.md`
- Create: `docs/plans/M31-03-cli-stdio-command.plan.md`
- Create: `docs/specs/M31-04-server-trace-audit.spec.md`
- Create: `docs/plans/M31-04-server-trace-audit.plan.md`
- Create: `docs/specs/M31-05-runtime-hardening-docs.spec.md`
- Create: `docs/plans/M31-05-runtime-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M31 Slices in Order

**Files:**
- Execute: `docs/plans/M31-01-stdio-session-runtime.plan.md`
- Execute: `docs/plans/M31-02-export-profile-bootstrap.plan.md`
- Execute: `docs/plans/M31-03-cli-stdio-command.plan.md`
- Execute: `docs/plans/M31-04-server-trace-audit.plan.md`
- Execute: `docs/plans/M31-05-runtime-hardening-docs.plan.md`

- [x] **Step 1: Add stdio session runtime**
- [x] **Step 2: Add export profile bootstrap**
- [ ] **Step 3: Add CLI stdio server command**
- [ ] **Step 4: Add server trace and audit summaries**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M31

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M31-mcp-stdio-server-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M31 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m31 mcp stdio server runtime"
```
