# M26-00 Runtime Observability Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M26 architecture baseline for observability evidence.

**Architecture:** Documentation-only slice covering `agent-telemetry`, `agent-core`, `agent-storage`, `agent-cli`, `agent-ui-bridge`, `agent-tui`, and `agent-desktop` boundaries.

**Tech Stack:** Markdown docs, existing architecture docs, telemetry/audit/replay contracts, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M26-runtime-observability-evidence.spec.md`
- Read: `docs/architecture/event-sourcing.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/specs/M12-02-telemetry.spec.md`
- Read: `docs/specs/M12-03-audit-export.spec.md`

- [x] **Step 1: Review milestone scope**

Expected: observability evidence scope connects telemetry, audit, trace, and replay without moving runtime logic into UI crates.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M26-00-runtime-observability-technical-design.spec.md`

- [x] **Step 1: Define evidence ownership and privacy rules**

Document crate boundaries, correlation keys, bundle schemas, replay semantics, and redaction requirements.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M26-00-runtime-observability-technical-design.spec.md`

- [x] **Step 1: Run focused documentation review**

Expected: technical design enables M26-01 through M26-05 without unresolved ownership gaps.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M26-00-runtime-observability-technical-design.spec.md`
- Add: `docs/plans/M26-00-runtime-observability-technical-design.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M26-00-runtime-observability-technical-design.spec.md docs/plans/M26-00-runtime-observability-technical-design.plan.md docs/milestones/README.md
git commit -m "docs: add m26 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
