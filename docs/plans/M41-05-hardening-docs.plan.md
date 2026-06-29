# M41-05 Hardening Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Document MCP sampling model bridge behavior and complete M41 verification.

**Architecture:** Documentation-only hardening over runtime changes implemented in `agent-core`.

**Tech Stack:** Markdown docs, Cargo workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M41-mcp-sampling-model-bridge.spec.md`
- Read: `docs/plans/M41-mcp-sampling-model-bridge.plan.md`
- Read: `README.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Read M41 specs and completed implementation slices**

Run:

```bash
sed -n '1,240p' docs/specs/M41-mcp-sampling-model-bridge.spec.md
sed -n '1,220p' docs/plans/M41-mcp-sampling-model-bridge.plan.md
```

Expected: scope, safe subset, and focused tests are clear.

### Task 2: Update Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M41-mcp-sampling-model-bridge.plan.md`

- [ ] **Step 1: Update README sampling model bridge usage**

Add a concise note that M41 can execute safe text-only sampling through local `agent-model` providers and that server hints remain advisory.

- [ ] **Step 2: Update milestone status and latest docs links**

Mark M41-01 through M41-05 complete only after implementation and verification are done.

### Task 3: Verify

- [ ] **Step 1: Run final M41 focused tests**

Run:

```bash
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_mapping
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_provider
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_redaction
CARGO_TARGET_DIR=target/codex-m41 cargo test -p agent-core mcp_sampling_model_guardrails
```

Expected: focused M41 tests pass.

- [ ] **Step 2: Run final workspace gate**

Run:

```bash
git diff --check
CARGO_TARGET_DIR=target/codex-m41 cargo fmt --all -- --check
CARGO_TARGET_DIR=target/codex-m41 cargo xtask ci
```

Expected: final gates pass.

### Task 4: Commit

- [ ] **Step 1: Commit M41-05**

Run:

```bash
git add README.md docs crates
git commit -m "feat: complete m41 mcp sampling model bridge"
```
