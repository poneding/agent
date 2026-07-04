# M11-00 Plugin Runtime Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M11 plugin runtime technical design baseline and decompose M11 into implementation slices.

**Architecture:** Keep the technical design in `docs/specs/` as an implementation guide. Slice plans must reference the design and preserve crate boundaries, policy routing, isolation rules, and verification gates.

**Tech Stack:** Markdown docs, existing `docs/specs`, `docs/plans`, `docs/milestones`, Cargo verification gates.

---

### Task 1: Write Technical Design

**Files:**
- Create: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`

- [x] **Step 1: Define plugin runtime architecture**

Cover crate ownership, manifest schema, version gates, isolation modes, process protocol, capability mapping, tool registration, hooks, WASM constraints, failure handling, security, and testing.

### Task 2: Decompose M11 Into Slices

**Files:**
- Create: `docs/specs/M11-01-plugin-manifest.spec.md`
- Create: `docs/specs/M11-02-process-plugin-host.spec.md`
- Create: `docs/specs/M11-03-plugin-capabilities.spec.md`
- Create: `docs/specs/M11-04-plugin-tools-hooks.spec.md`
- Create: `docs/specs/M11-05-wasm-prototype.spec.md`
- Create: `docs/plans/M11-01-plugin-manifest.plan.md`
- Create: `docs/plans/M11-02-process-plugin-host.plan.md`
- Create: `docs/plans/M11-03-plugin-capabilities.plan.md`
- Create: `docs/plans/M11-04-plugin-tools-hooks.plan.md`
- Create: `docs/plans/M11-05-wasm-prototype.plan.md`

- [x] **Step 1: Generate slice specs and plans**

Each slice file names concrete ownership, public contracts, tests, verification commands, and commit boundaries.

### Task 3: Update Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M11-plugin-runtime.plan.md`

- [x] **Step 1: Point active work at M11-01**

Set current implementation focus to `M11-01-plugin-manifest`.

- [x] **Step 2: Run verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 3: Commit**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: decompose m11 plugin runtime"
```

Expected: commit succeeds.
