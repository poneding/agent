# M15 Config-Driven Turn Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M15 through independently testable slices that connect effective runtime config to provider resolution, invocation policy, configured turn execution, CLI prompt runtime, and persisted event replay.

**Architecture:** Treat this file as the M15 coordination plan. Each slice owns its own spec and plan, lands in a focused commit, and follows `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, async trait model providers, serde contracts, in-memory fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M15 Technical Design

**Files:**
- Create: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`
- Create: `docs/plans/M15-00-config-driven-turn-execution-technical-design.plan.md`

- [x] **Step 1: Write the M15 technical design baseline**

Define provider resolution, capability checks, invocation retry/fallback, configured turn runner boundaries, CLI prompt execution, JSON output, persistence, privacy rules, and verification gates.

- [x] **Step 2: Create the M15 design execution plan**

Record the documentation work and slice decomposition tasks.

### Task 2: Decompose M15 Into Slice Specs And Plans

**Files:**
- Create: `docs/specs/M15-01-model-runtime-resolver.spec.md`
- Create: `docs/plans/M15-01-model-runtime-resolver.plan.md`
- Create: `docs/specs/M15-02-invocation-policy-runner.spec.md`
- Create: `docs/plans/M15-02-invocation-policy-runner.plan.md`
- Create: `docs/specs/M15-03-configured-turn-runner.spec.md`
- Create: `docs/plans/M15-03-configured-turn-runner.plan.md`
- Create: `docs/specs/M15-04-cli-prompt-runtime.spec.md`
- Create: `docs/plans/M15-04-cli-prompt-runtime.plan.md`
- Create: `docs/specs/M15-05-persisted-turn-events.spec.md`
- Create: `docs/plans/M15-05-persisted-turn-events.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec defines purpose, scope, architecture, public contracts, state and failure handling, security and privacy constraints, tests, acceptance criteria, and completion signal.

- [x] **Step 2: Create slice plans**

Each slice plan defines exact files, RED tests, focused verification commands, workspace verification gates, status updates, and commit boundaries.

### Task 3: Sync Documentation Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

- [x] **Step 1: Point active work at M15-01**

Update README and milestone navigation so the current spec and plan are:

```text
docs/specs/M15-01-model-runtime-resolver.spec.md
docs/plans/M15-01-model-runtime-resolver.plan.md
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
git commit -m "docs: plan m15 config-driven turn execution"
```

Expected: commit succeeds with M15 planning docs.

### Task 4: Implement M15 Slices In Order

**Files:**
- Execute: `docs/plans/M15-01-model-runtime-resolver.plan.md`
- Execute: `docs/plans/M15-02-invocation-policy-runner.plan.md`
- Execute: `docs/plans/M15-03-configured-turn-runner.plan.md`
- Execute: `docs/plans/M15-04-cli-prompt-runtime.plan.md`
- Execute: `docs/plans/M15-05-persisted-turn-events.plan.md`

- [x] **Step 1: Implement M15-01 model runtime resolver**

Complete provider selection and capability checks, then commit `feat: add m15 model runtime resolver`.

- [x] **Step 2: Implement M15-02 invocation policy runner**

Complete retry and fallback invocation execution, then commit `feat: add m15 invocation policy runner`.

- [x] **Step 3: Implement M15-03 configured turn runner**

Complete config-driven turn execution through `TurnEngine`, then commit `feat: add m15 configured turn runner`.

- [x] **Step 4: Implement M15-04 CLI prompt runtime**

Complete text-mode CLI prompt execution through configured runtime, then commit `feat: add m15 cli prompt runtime`.

- [ ] **Step 5: Implement M15-05 persisted turn events**

Complete persistence and JSON prompt event replay coverage, then commit `feat: add m15 persisted turn events`.

### Task 5: Complete M15

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Mark M15 complete**

Update M15 planning status and advance active focus to the next milestone.

- [ ] **Step 3: Commit M15 completion status**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: mark m15 config-driven turn execution complete"
```

Expected: commit succeeds.
