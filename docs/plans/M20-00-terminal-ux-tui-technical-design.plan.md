# M20-00 Terminal UX and TUI Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M20 architecture baseline for the `agent` binary and Ratatui-powered terminal REPL.

**Architecture:** This is a documentation-only slice. It defines `ruse-cli` entrypoint ownership, `ruse-tui` rendering/input ownership, UI bridge boundaries, TTY fallback behavior, and terminal cleanup requirements.

**Tech Stack:** Markdown docs, Ratatui/Crossterm/tui-textarea design references, existing UI bridge docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-terminal-ux-tui-repl.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/specs/M06-json-ui-bridge.spec.md`
- Read: `docs/specs/M19-00-interactive-approval-technical-design.spec.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M20-terminal-ux-tui-repl.spec.md
```

Expected: milestone scope defines `agent` binary behavior, `ruse-tui`, Ratatui shell, panels, and fallback modes.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document binary command model, crate boundaries, app state, terminal lifecycle, runtime bridge client, rendering strategy, security, and tests.

- [ ] **Step 2: Self-review design**

Check that batch/JSON modes remain script-safe, raw terminal mode is restored, and side-effecting TUI actions require explicit focus and intent.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`
- Add: `docs/plans/M20-00-terminal-ux-tui-technical-design.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md docs/plans/M20-00-terminal-ux-tui-technical-design.plan.md docs/plans/M20-terminal-ux-tui-repl.plan.md
git commit -m "docs: add m20 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
