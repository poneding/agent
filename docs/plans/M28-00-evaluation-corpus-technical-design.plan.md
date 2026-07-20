# M28-00 Evaluation Corpus Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M28 suite/corpus/gate design baseline.

**Architecture:** `ruse-eval` owns suite expansion. `ruse-cli` and `xtask` consume typed contracts.

**Tech Stack:** Markdown specs/plans, existing M27 eval contracts, Cargo workspace conventions.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M28-evaluation-corpus-quality-gates.spec.md`
- Read: `docs/architecture/README.md`

- [x] **Step 1: Review architecture boundaries**

Expected: suite logic belongs in `ruse-eval`, not CLI or Desktop.

### Task 2: Write Design and Slice Plans

**Files:**
- Create: `docs/specs/M28-00-evaluation-corpus-technical-design.spec.md`
- Create: `docs/plans/M28-00-evaluation-corpus-technical-design.plan.md`

- [x] **Step 1: Define suite/corpus/gate architecture**

Expected: design names ownership, path safety, privacy, and verification gates.

### Task 3: Update Milestone Index

**Files:**
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M28 active**

Expected: milestone status, index, slice index, and active work all reference M28.
