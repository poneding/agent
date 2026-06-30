# M50 MCP HTTP Background Stream Supervision Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add bounded background supervision foundations for MCP streamable HTTP GET streams over the M49 reconnect runtime.

**Architecture:** `agent-mcp` owns supervisor contracts, runner, cancellation, and safe reports. Scheduler/UI crates consume typed DTOs only.

**Tech Stack:** Rust 2024 edition, async-trait, serde, fixture transports, scheduler contracts, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M50 Technical Design

**Files:**
- Create: `docs/specs/M50-00-mcp-http-background-stream-supervision-technical-design.spec.md`
- Create: `docs/plans/M50-00-mcp-http-background-stream-supervision-technical-design.plan.md`

- [x] **Step 1: Write the M50 technical design baseline**

Define crate ownership, supervisor lifecycle, reconnect composition, cancellation/lease boundaries, redaction behavior, and verification gates.

### Task 2: Decompose M50 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M50-01-stream-supervisor-contracts.spec.md`
- Create: `docs/plans/M50-01-stream-supervisor-contracts.plan.md`
- Create: `docs/specs/M50-02-supervisor-runner.spec.md`
- Create: `docs/plans/M50-02-supervisor-runner.plan.md`
- Create: `docs/specs/M50-03-cancellation-lease-boundary.spec.md`
- Create: `docs/plans/M50-03-cancellation-lease-boundary.plan.md`
- Create: `docs/specs/M50-04-supervisor-reports.spec.md`
- Create: `docs/plans/M50-04-supervisor-reports.plan.md`
- Create: `docs/specs/M50-05-hardening-docs.spec.md`
- Create: `docs/plans/M50-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M50 Slices in Order

**Files:**
- Execute: `docs/plans/M50-01-stream-supervisor-contracts.plan.md`
- Execute: `docs/plans/M50-02-supervisor-runner.plan.md`
- Execute: `docs/plans/M50-03-cancellation-lease-boundary.plan.md`
- Execute: `docs/plans/M50-04-supervisor-reports.plan.md`
- Execute: `docs/plans/M50-05-hardening-docs.plan.md`

- [x] **Step 1: Add stream supervisor contracts**
- [x] **Step 2: Add bounded supervisor runner**
- [x] **Step 3: Add cancellation and lease boundary**
- [x] **Step 4: Add redacted supervisor reports**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M50

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M50-mcp-http-background-stream-supervision.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m50 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M50 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m50 mcp http background stream supervision"
```
