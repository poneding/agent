# M42-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP sampling stdio dispatch behavior and complete M42 verification.

**Architecture:** Documentation-only hardening over runtime changes implemented in `agent-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M42-mcp-sampling-stdio-dispatch.spec.md`
- Read: `docs/plans/M42-mcp-sampling-stdio-dispatch.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Read M42 specs and completed implementation slices**

Expected: scope, safe subset, and focused tests are clear.

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M42-mcp-sampling-stdio-dispatch.plan.md`

- [ ] **Step 1: Update README sampling stdio dispatch usage**

Add a concise note that M42 can dispatch fixture-grade `sampling/createMessage` JSON-RPC request frames through local sampling providers.

- [ ] **Step 2: Update milestone status and latest docs links**

Mark M42-01 through M42-05 complete only after implementation and verification are done.

### Task 3: Verify

- [ ] **Step 1: Run final M42 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_contracts
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_dispatch
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_validation
CARGO_TARGET_DIR=target/codex-m42 cargo test -p agent-mcp mcp_sampling_stdio_redaction
```

Expected: focused M42 tests pass.

- [ ] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m42 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m42 cargo xtask ci
```

Expected: final gates pass.

### Task 4: Commit

- [ ] **Step 1: Commit M42-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m42 mcp sampling stdio dispatch"
```
