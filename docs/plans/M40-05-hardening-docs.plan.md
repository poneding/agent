# M40-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP sampling runtime behavior and complete M40 verification.

**Architecture:** Documentation-only hardening over runtime changes already implemented in `agent-mcp`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M40-mcp-sampling-runtime.spec.md`
- Read: `docs/plans/M40-mcp-sampling-runtime.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [x] **Step 1: Read M40 specs and completed implementation slices**

Run:

```bash
sed -n '1,240p' docs/specs/M40-mcp-sampling-runtime.spec.md
sed -n '1,220p' docs/plans/M40-mcp-sampling-runtime.plan.md
```

Expected: scope, safe subset, and focused tests are clear.

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M40-mcp-sampling-runtime.plan.md`

- [x] **Step 1: Update README sampling usage**

Add a concise MCP Sampling section that states the safe subset, shows fixture-style usage, and notes that prompts/responses are not included in gateway reports.

- [x] **Step 2: Update milestone status and latest docs links**

Mark M40-01 through M40-05 complete only after implementation and verification are done. Set the next implementation slice to pending next milestone planning.

### Task 3: Verify

- [x] **Step 1: Run final M40 focused tests**

Verified:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_gateway_sampling
```

Run:

```bash
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_contracts
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_sampling_validation
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_client_session_sampling
CARGO_TARGET_DIR=target/codex-m40 cargo test -p agent-mcp mcp_gateway_sampling
```

Expected: focused M40 tests pass.

- [x] **Step 2: Run final workspace gate**

Verified:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo xtask ci
```

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m40 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m40 cargo xtask ci
```

Expected: final gates pass.

### Task 4: Commit

- [x] **Step 1: Commit M40-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m40 mcp sampling runtime"
```

Expected: commit succeeds with source, tests, and documentation together.
