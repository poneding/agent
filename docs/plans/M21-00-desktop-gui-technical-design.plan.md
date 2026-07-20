# M21-00 Desktop GUI Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M21 architecture baseline for a runnable Tauri Desktop MVP.

**Architecture:** This is a documentation-only slice. It defines Tauri command boundaries, frontend state ownership, runtime bridge sessions, settings/secrets behavior, and Desktop smoke testing strategy.

**Tech Stack:** Markdown docs, Tauri 2 design notes, existing `ruse-desktop` and `ruse-ui-bridge` contracts, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M21-desktop-gui-mvp.spec.md`
- Read: `docs/specs/M10-desktop-preview.spec.md`
- Read: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`
- Read: `docs/architecture/crate-boundaries.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M21-desktop-gui-mvp.spec.md
```

Expected: milestone scope defines runnable Tauri Desktop app over shared runtime bridge.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M21-00-desktop-gui-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document Tauri app structure, command boundary, runtime session, view layout, settings, errors, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that frontend does not call runtime internals directly and secret values never enter frontend state.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M21-00-desktop-gui-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M21-00-desktop-gui-technical-design.spec.md`
- Add: `docs/plans/M21-00-desktop-gui-technical-design.plan.md`
- Modify: `docs/plans/M21-desktop-gui-mvp.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M21-00-desktop-gui-technical-design.spec.md docs/plans/M21-00-desktop-gui-technical-design.plan.md docs/plans/M21-desktop-gui-mvp.plan.md
git commit -m "docs: add m21 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
