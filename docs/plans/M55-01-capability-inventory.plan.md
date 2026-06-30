# M55-01 Capability Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a repository-grounded mainline capability baseline for M56-M66 planning.

**Architecture:** Documentation-only slice. Read crates, tests, docs, README command surfaces, and roadmap state; write the product baseline under `docs/roadmap/`.

**Tech Stack:** Rust 2024 edition repository, Markdown docs, `rg`, `cargo metadata`, `git diff --check`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M55-01-capability-inventory.spec.md`
- Read: `README.md`
- Read: `docs/PRD.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `docs/architecture/README.md`

- [ ] **Step 1: Review required baseline domains**

Run:

```bash
sed -n '1,260p' docs/specs/M55-01-capability-inventory.spec.md
```

Expected: required domains, status vocabulary, and evidence contract are clear.

### Task 2: Inventory Repository Evidence

**Files:**
- Read: `Cargo.toml`
- Read: `README.md`
- Read: `crates/*/Cargo.toml`
- Read: relevant `crates/*/src/*.rs`
- Read: relevant `crates/*/tests/*.rs`
- Read: `docs/eval/` if present
- Read: `xtask/`

- [ ] **Step 1: List crates and commands**

Run:

```bash
cargo metadata --no-deps --format-version 1
rg --files crates xtask docs | sort
rg "agent-cli|cargo xtask|eval|desktop|mcp|replay|checkpoint|approval|provider" README.md docs crates xtask
```

Expected: enough evidence to map domains to concrete implementation and verification paths.

### Task 3: Write Baseline Document

**Files:**
- Create: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`

- [ ] **Step 1: Create capability baseline**

Include:

- status legend
- executive summary
- capability table with required domains
- cross-cutting product risks
- immediate implications for M56-M66

- [ ] **Step 2: Mark uncertain items explicitly**

Use `Unknown` only when paired with a concrete verification command or file path.

### Task 4: Verify and Commit

**Files:**
- Create: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Modify: `docs/plans/M55-01-capability-inventory.plan.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Run docs verification**

Run:

```bash
git diff --check -- docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md docs/plans/M55-01-capability-inventory.plan.md docs/milestones/README.md
```

Expected: no whitespace errors.

- [ ] **Step 2: Commit**

Run exact-path staging only:

```bash
git add docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md docs/plans/M55-01-capability-inventory.plan.md docs/milestones/README.md
git commit -m "docs: add m55 capability baseline"
```

Expected: commit contains only M55-01 baseline docs and status updates.

### Completion Checklist

- [ ] Required domains are present.
- [ ] Evidence points to real files, commands, tests, or docs.
- [ ] Gaps are mapped to M56-M66.
- [ ] MCP is represented as parked extension depth.
- [ ] `git diff --check` passes for changed docs.
