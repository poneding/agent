# M12 Production Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M12 through independently testable slices for migrations, telemetry, audit export, memory portability, platform reports, and benchmarks.

**Architecture:** Treat this file as the M12 coordination plan, not a single implementation batch. Each slice owns its own spec and plan, lands in its own focused commit, and preserves the crate boundaries defined by `docs/specs/M12-00-production-hardening-technical-design.spec.md`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible durable contracts, thiserror error types, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M12 Technical Design

**Files:**
- Create: `docs/specs/M12-00-production-hardening-technical-design.spec.md`
- Create: `docs/plans/M12-00-production-hardening-technical-design.plan.md`

- [x] **Step 1: Write the M12 technical design baseline**

Define crate ownership, dependency direction, migration rules, telemetry rules, audit export rules, memory archive rules, platform report rules, benchmark rules, privacy constraints, and verification gates.

- [x] **Step 2: Create the M12 design execution plan**

Record the documentation work and slice decomposition tasks.

### Task 2: Decompose M12 Into Slice Specs And Plans

**Files:**
- Create: `docs/specs/M12-01-schema-migration.spec.md`
- Create: `docs/plans/M12-01-schema-migration.plan.md`
- Create: `docs/specs/M12-02-telemetry.spec.md`
- Create: `docs/plans/M12-02-telemetry.plan.md`
- Create: `docs/specs/M12-03-audit-export.spec.md`
- Create: `docs/plans/M12-03-audit-export.plan.md`
- Create: `docs/specs/M12-04-memory-portability.spec.md`
- Create: `docs/plans/M12-04-memory-portability.plan.md`
- Create: `docs/specs/M12-05-platform-capability-report.spec.md`
- Create: `docs/plans/M12-05-platform-capability-report.plan.md`
- Create: `docs/specs/M12-06-benchmarks.spec.md`
- Create: `docs/plans/M12-06-benchmarks.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec defines purpose, scope, architecture, file layout, public contracts, state and failure handling, security and privacy constraints, tests, acceptance criteria, and completion signal.

- [x] **Step 2: Create slice plans**

Each slice plan defines exact files, RED tests, focused verification commands, workspace verification gates, status updates, and commit boundaries.

### Task 3: Sync Documentation Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-production-hardening.plan.md`

- [x] **Step 1: Point active work at M12-01**

Update README and milestone navigation so the current spec and plan are:

```text
docs/specs/M12-01-schema-migration.spec.md
docs/plans/M12-01-schema-migration.plan.md
```

- [x] **Step 2: Run workspace verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 3: Commit documentation decomposition**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: decompose m12 production hardening"
```

Expected: commit succeeds.

### Task 4: Implement M12-01 Schema Migration

**Files:**
- Read: `docs/specs/M12-01-schema-migration.spec.md`
- Execute: `docs/plans/M12-01-schema-migration.plan.md`

- [x] **Step 1: Complete M12-01 using TDD**

Follow `docs/plans/M12-01-schema-migration.plan.md` exactly: write RED tests, verify failure, implement the migration module, verify focused tests, run workspace gates, update status, and commit.

### Task 5: Implement Remaining M12 Slices In Order

**Files:**
- Execute: `docs/plans/M12-02-telemetry.plan.md`
- Execute: `docs/plans/M12-03-audit-export.plan.md`
- Execute: `docs/plans/M12-04-memory-portability.plan.md`
- Execute: `docs/plans/M12-05-platform-capability-report.plan.md`
- Execute: `docs/plans/M12-06-benchmarks.plan.md`

- [ ] **Step 1: Implement M12-02 telemetry**

Complete the telemetry crate and commit `feat: add m12 telemetry`.

- [ ] **Step 2: Implement M12-03 audit export**

Complete audit export bundles and commit `feat: add m12 audit export`.

- [ ] **Step 3: Implement M12-04 memory portability**

Complete memory archives and commit `feat: add m12 memory portability`.

- [ ] **Step 4: Implement M12-05 platform capability report**

Complete sandbox capability reports and commit `feat: add m12 platform report`.

- [ ] **Step 5: Implement M12-06 benchmarks**

Complete deterministic benchmark smoke scenarios and commit `feat: add m12 benchmarks`.

### Task 6: Complete M12

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-production-hardening.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Mark M12 complete**

Update M12 planning status and advance active focus to the next milestone.

- [ ] **Step 3: Commit M12 completion status**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: mark m12 production hardening complete"
```

Expected: commit succeeds.
