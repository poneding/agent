# M12-00 Production Hardening Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M12 production-hardening technical design baseline and decompose M12 into implementation slices.

**Architecture:** Keep production-hardening guidance in `docs/specs/` as a stable baseline. Slice plans must preserve crate boundaries, typed durable contracts, redaction, migration compatibility, and verification gates.

**Tech Stack:** Markdown docs, existing `docs/specs`, `docs/plans`, `docs/milestones`, Cargo verification gates.

---

### Task 1: Write Technical Design

**Files:**
- Create: `docs/specs/M12-00-production-hardening-technical-design.spec.md`

- [x] **Step 1: Define production hardening architecture**

Cover migrations, telemetry, audit export, memory portability, platform reports, benchmarks, crate ownership, security, privacy, and tests.

### Task 2: Decompose M12 Into Slices

**Files:**
- Create: `docs/specs/M12-01-schema-migration.spec.md`
- Create: `docs/specs/M12-02-telemetry.spec.md`
- Create: `docs/specs/M12-03-audit-export.spec.md`
- Create: `docs/specs/M12-04-memory-portability.spec.md`
- Create: `docs/specs/M12-05-platform-capability-report.spec.md`
- Create: `docs/specs/M12-06-benchmarks.spec.md`
- Create: `docs/plans/M12-01-schema-migration.plan.md`
- Create: `docs/plans/M12-02-telemetry.plan.md`
- Create: `docs/plans/M12-03-audit-export.plan.md`
- Create: `docs/plans/M12-04-memory-portability.plan.md`
- Create: `docs/plans/M12-05-platform-capability-report.plan.md`
- Create: `docs/plans/M12-06-benchmarks.plan.md`

- [x] **Step 1: Generate slice specs and plans**

Each slice file names concrete ownership, public contracts, tests, verification commands, and commit boundaries.

### Task 3: Update Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-production-hardening.plan.md`

- [x] **Step 1: Point active work at M12-01**

Set current implementation focus to `M12-01-schema-migration`.

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
git commit -m "docs: decompose m12 production hardening"
```

Expected: commit succeeds.
