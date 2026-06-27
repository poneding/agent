# M19-00 Interactive Approval Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M19 architecture baseline for durable interactive approvals and safe local actions.

**Architecture:** This is a documentation-only slice. It defines `agent-policy` lease ownership, `agent-core` pending approval and resume behavior, `agent-ui-bridge` approval schemas, and CLI approval prompt boundaries.

**Tech Stack:** Markdown docs, existing policy/tool/runtime architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M19-interactive-approval-safe-actions.spec.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/specs/M18-00-agent-tool-loop-technical-design.spec.md`
- Read: `docs/specs/M06-json-ui-bridge.spec.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M19-interactive-approval-safe-actions.spec.md
```

Expected: milestone scope defines durable approvals, previews, leases, CLI prompts, and recovery.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document pending approval records, bridge schemas, preview payloads, lease rules, resume behavior, CLI behavior, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that approvals never bypass sandboxing, previews redact secrets, and expired or stale responses cannot execute tools.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`
- Add: `docs/plans/M19-00-interactive-approval-technical-design.plan.md`
- Modify: `docs/plans/M19-interactive-approval-safe-actions.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M19-00-interactive-approval-technical-design.spec.md docs/plans/M19-00-interactive-approval-technical-design.plan.md docs/plans/M19-interactive-approval-safe-actions.plan.md
git commit -m "docs: add m19 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
