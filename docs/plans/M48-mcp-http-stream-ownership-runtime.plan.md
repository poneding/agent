# M48 MCP HTTP Stream Ownership Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the typed runtime foundation for opening and owning resumable MCP streamable HTTP GET streams.

**Architecture:** `agent-mcp` owns GET stream request contracts, stream-open transport boundaries, resume headers, bounded event drain, redaction, and tests. CLI/Desktop remain consumers of typed runtime contracts.

**Tech Stack:** Rust 2024 edition, async-trait, serde JSON, reqwest, HTTP headers, SSE parsing, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Establish M48 Technical Design

**Files:**
- Create: `docs/specs/M48-00-mcp-http-stream-ownership-technical-design.spec.md`
- Create: `docs/plans/M48-00-mcp-http-stream-ownership-technical-design.plan.md`

- [x] **Step 1: Write the M48 technical design baseline**

Define crate ownership, stream request contracts, stream transport boundary, resume header behavior, bounded drain rules, redaction behavior, and verification gates.

### Task 2: Decompose M48 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M48-01-http-stream-request-contracts.spec.md`
- Create: `docs/plans/M48-01-http-stream-request-contracts.plan.md`
- Create: `docs/specs/M48-02-reqwest-get-sse-stream-open.spec.md`
- Create: `docs/plans/M48-02-reqwest-get-sse-stream-open.plan.md`
- Create: `docs/specs/M48-03-stream-resume-snapshot.spec.md`
- Create: `docs/plans/M48-03-stream-resume-snapshot.plan.md`
- Create: `docs/specs/M48-04-bounded-stream-event-drain.spec.md`
- Create: `docs/plans/M48-04-bounded-stream-event-drain.plan.md`
- Create: `docs/specs/M48-05-hardening-docs.spec.md`
- Create: `docs/plans/M48-05-hardening-docs.plan.md`

- [x] **Step 1: Create slice specs**
- [x] **Step 2: Create slice plans**

### Task 3: Implement M48 Slices in Order

**Files:**
- Execute: `docs/plans/M48-01-http-stream-request-contracts.plan.md`
- Execute: `docs/plans/M48-02-reqwest-get-sse-stream-open.plan.md`
- Execute: `docs/plans/M48-03-stream-resume-snapshot.plan.md`
- Execute: `docs/plans/M48-04-bounded-stream-event-drain.plan.md`
- Execute: `docs/plans/M48-05-hardening-docs.plan.md`

- [x] **Step 1: Add GET stream request contracts**
- [x] **Step 2: Add reqwest GET SSE stream open**
- [x] **Step 3: Add initialized stream resume snapshots**
- [x] **Step 4: Add bounded stream event drain**
- [ ] **Step 5: Add hardening docs and final verification**

### Task 4: Complete M48

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M48-mcp-http-stream-ownership-runtime.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m48 cargo xtask ci
```

Expected: all checks pass.

- [ ] **Step 2: Mark M48 complete and commit**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m48 mcp http stream ownership runtime"
```
