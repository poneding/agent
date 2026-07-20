# M16-00 Prompt Runtime Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M16 architecture baseline for dynamic prompt management.

**Architecture:** This is a documentation-only slice. It defines `ruse-context` ownership for prompt runtime contracts, `ruse-core` persistence boundaries, UI-safe prompt summaries, and replay/privacy rules before implementation slices begin.

**Tech Stack:** Markdown docs, existing architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M16-prompt-runtime-dynamic-management.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/configuration.md`
- Read: `docs/architecture/event-sourcing.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M16-prompt-runtime-dynamic-management.spec.md
```

Expected: milestone scope defines dynamic prompt profiles, overlays, diagnostics, and snapshots.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document prompt profile data, layer sources, overlay rules, prompt snapshots, UI summaries, failure handling, privacy, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that hidden prompts, secret-like values, project prompt injection, memory sensitivity, and replayability have explicit handling.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: formatting, clippy, and workspace tests pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M16-00-prompt-runtime-technical-design.spec.md`
- Add: `docs/plans/M16-00-prompt-runtime-technical-design.plan.md`
- Modify: `docs/plans/M16-prompt-runtime-dynamic-management.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M16-00-prompt-runtime-technical-design.spec.md docs/plans/M16-00-prompt-runtime-technical-design.plan.md docs/plans/M16-prompt-runtime-dynamic-management.plan.md
git commit -m "docs: add m16 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
