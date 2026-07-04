# M18-00 Agent Tool Loop Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M18 architecture baseline for comprehensive model-tool-observation turns.

**Architecture:** This is a documentation-only slice. It defines `agent-core` loop ownership, tool-call accumulation, policy/sandbox flow, observation context, loop guards, idempotency, and recovery behavior.

**Tech Stack:** Markdown docs, existing tool/policy/runtime architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M18-agent-tool-loop-runtime.spec.md`
- Read: `docs/specs/M02-tool-policy-audit.spec.md`
- Read: `docs/specs/M03-local-coding-loop.spec.md`
- Read: `docs/specs/M17-live-model-runtime-streaming.spec.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M18-agent-tool-loop-runtime.spec.md
```

Expected: milestone scope defines comprehensive agent tool loop behavior and leaves interactive approval UI to M19.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document loop state machine, tool-call accumulator, observation messages, guards, idempotency, recovery, security, and test strategy.

- [ ] **Step 2: Self-review design**

Check that every side effect routes through policy, all tool output is treated as untrusted, and runaway loops have explicit guards.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md`
- Add: `docs/plans/M18-00-agent-tool-loop-technical-design.plan.md`
- Modify: `docs/plans/M18-agent-tool-loop-runtime.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M18-00-agent-tool-loop-technical-design.spec.md docs/plans/M18-00-agent-tool-loop-technical-design.plan.md docs/plans/M18-agent-tool-loop-runtime.plan.md
git commit -m "docs: add m18 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
