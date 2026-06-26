# M14 Runtime Configuration And Secrets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M14 through independently testable slices for secret resolution, config loading, config layering, CLI diagnostics, and UI config snapshots.

**Architecture:** Treat this file as the M14 coordination plan. Each slice owns its own spec and plan, lands in a focused commit, and follows `docs/specs/M14-00-runtime-configuration-secrets-technical-design.spec.md`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde contracts, TOML config parsing, thiserror error types, deterministic fixture tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M14 Technical Design

**Files:**
- Create: `docs/specs/M14-00-runtime-configuration-secrets-technical-design.spec.md`
- Create: `docs/plans/M14-00-runtime-configuration-secrets-technical-design.plan.md`

- [x] **Step 1: Write the M14 technical design baseline**

Define secret resolver boundaries, redacted secret values, config schema version 1, TOML parsing, layer precedence, CLI diagnostics, UI snapshots, privacy rules, and verification gates.

- [x] **Step 2: Create the M14 design execution plan**

Record the documentation work and slice decomposition tasks.

### Task 2: Decompose M14 Into Slice Specs And Plans

**Files:**
- Create: `docs/specs/M14-01-secret-resolver.spec.md`
- Create: `docs/plans/M14-01-secret-resolver.plan.md`
- Create: `docs/specs/M14-02-config-file-loader.spec.md`
- Create: `docs/plans/M14-02-config-file-loader.plan.md`
- Create: `docs/specs/M14-03-config-layering.spec.md`
- Create: `docs/plans/M14-03-config-layering.plan.md`
- Create: `docs/specs/M14-04-cli-config-diagnostics.spec.md`
- Create: `docs/plans/M14-04-cli-config-diagnostics.plan.md`
- Create: `docs/specs/M14-05-ui-config-snapshot.spec.md`
- Create: `docs/plans/M14-05-ui-config-snapshot.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec defines purpose, scope, architecture, public contracts, state and failure handling, security and privacy constraints, tests, acceptance criteria, and completion signal.

- [x] **Step 2: Create slice plans**

Each slice plan defines exact files, RED tests, focused verification commands, workspace verification gates, status updates, and commit boundaries.

### Task 3: Sync Documentation Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`

- [x] **Step 1: Point active work at M14-01**

Update README and milestone navigation so the current spec and plan are:

```text
docs/specs/M14-01-secret-resolver.spec.md
docs/plans/M14-01-secret-resolver.plan.md
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
git commit -m "docs: plan m14 runtime configuration secrets"
```

Expected: commit succeeds with M14 planning docs.

### Task 4: Implement M14 Slices In Order

**Files:**
- Execute: `docs/plans/M14-01-secret-resolver.plan.md`
- Execute: `docs/plans/M14-02-config-file-loader.plan.md`
- Execute: `docs/plans/M14-03-config-layering.plan.md`
- Execute: `docs/plans/M14-04-cli-config-diagnostics.plan.md`
- Execute: `docs/plans/M14-05-ui-config-snapshot.plan.md`

- [x] **Step 1: Implement M14-01 secret resolver**

Complete redacted secret value and resolver contracts, then commit `feat: add m14 secret resolver`.

- [x] **Step 2: Implement M14-02 config file loader**

Complete schema version 1 TOML loading and validation, then commit `feat: add m14 runtime config loader`.

- [x] **Step 3: Implement M14-03 config layering**

Complete deterministic model config layering and overrides, then commit `feat: add m14 config layering`.

- [ ] **Step 4: Implement M14-04 CLI config diagnostics**

Complete CLI config-check flags and diagnostics, then commit `feat: add m14 cli config diagnostics`.

- [ ] **Step 5: Implement M14-05 UI config snapshot**

Complete UI/Desktop config snapshot contracts, then commit `feat: add m14 ui config snapshot`.

### Task 5: Complete M14

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Mark M14 complete**

Update M14 planning status and advance active focus to the next milestone.

- [ ] **Step 3: Commit M14 completion status**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: mark m14 runtime configuration secrets complete"
```

Expected: commit succeeds.
