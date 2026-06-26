# M15-00 Config-Driven Turn Execution Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M15 architecture baseline for config-driven turn execution.

**Architecture:** This is a documentation-only slice. It defines `agent-core::model_runtime` ownership, dependency direction, CLI boundaries, persistence boundaries, and privacy constraints before source implementation begins.

**Tech Stack:** Markdown docs, existing architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-config-driven-turn-execution.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `docs/architecture/configuration.md`
- Read: `docs/architecture/event-sourcing.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,260p' docs/specs/M15-config-driven-turn-execution.spec.md
```

Expected: milestone scope defines config-driven turn execution and leaves live HTTP transport to later work.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document provider resolution, invocation policy runner, configured turn runner, CLI prompt boundary, persistence, privacy, and testing strategy.

- [ ] **Step 2: Self-review design**

Check for missing ownership decisions, provider transport ambiguity, secret leakage risks, and test gaps.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`
- Add: `docs/plans/M15-00-config-driven-turn-execution-technical-design.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md docs/plans/M15-00-config-driven-turn-execution-technical-design.plan.md
git commit -m "docs: add m15 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
