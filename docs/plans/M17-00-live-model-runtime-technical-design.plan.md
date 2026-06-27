# M17-00 Live Model Runtime Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M17 architecture baseline for live OpenAI-compatible HTTP execution and true model streaming.

**Architecture:** This is a documentation-only slice. It defines `agent-model` streaming provider ownership, reqwest transport boundaries, `agent-core` stream consumption, CLI live registration, and no-network test strategy.

**Tech Stack:** Markdown docs, existing model/config architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M17-live-model-runtime-streaming.spec.md`
- Read: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`
- Read: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`
- Read: `docs/architecture/crate-boundaries.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M17-live-model-runtime-streaming.spec.md
```

Expected: milestone scope defines live transport, true streaming, CLI output, and fixture-only verification.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M17-00-live-model-runtime-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document stream trait migration, reqwest transport, secret injection, retry/fallback/cancellation, CLI behavior, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that provider-specific code stays in `agent-model`, core sees only provider-neutral stream events, and tests do not require live network.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M17-00-live-model-runtime-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M17-00-live-model-runtime-technical-design.spec.md`
- Add: `docs/plans/M17-00-live-model-runtime-technical-design.plan.md`
- Modify: `docs/plans/M17-live-model-runtime-streaming.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M17-00-live-model-runtime-technical-design.spec.md docs/plans/M17-00-live-model-runtime-technical-design.plan.md docs/plans/M17-live-model-runtime-streaming.plan.md
git commit -m "docs: add m17 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
