# M13 Real Model Providers And Runtime Configuration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete M13 through independently testable slices for provider config, registry, invocation policy, OpenAI-compatible adapter, and CLI model selection.

**Architecture:** Treat this file as the M13 coordination plan. Each slice owns its own spec and plan, lands in a focused commit, and follows `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde contracts, async-trait provider interface, thiserror error types, fixture transports, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M13 Technical Design

**Files:**
- Create: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`
- Create: `docs/plans/M13-00-real-model-providers-config-technical-design.plan.md`

- [x] **Step 1: Write the M13 technical design baseline**

Define provider config, secret references, registry, capability negotiation, invocation policy, OpenAI-compatible adapter boundaries, CLI/Desktop boundaries, privacy rules, and verification gates.

- [x] **Step 2: Create the M13 design execution plan**

Record the documentation work and slice decomposition tasks.

### Task 2: Decompose M13 Into Slice Specs And Plans

**Files:**
- Create: `docs/specs/M13-01-provider-config.spec.md`
- Create: `docs/plans/M13-01-provider-config.plan.md`
- Create: `docs/specs/M13-02-provider-registry-capabilities.spec.md`
- Create: `docs/plans/M13-02-provider-registry-capabilities.plan.md`
- Create: `docs/specs/M13-03-invocation-policy.spec.md`
- Create: `docs/plans/M13-03-invocation-policy.plan.md`
- Create: `docs/specs/M13-04-openai-compatible-adapter.spec.md`
- Create: `docs/plans/M13-04-openai-compatible-adapter.plan.md`
- Create: `docs/specs/M13-05-cli-model-selection.spec.md`
- Create: `docs/plans/M13-05-cli-model-selection.plan.md`

- [x] **Step 1: Create slice specs**

Each slice spec defines purpose, scope, architecture, public contracts, state and failure handling, security and privacy constraints, tests, acceptance criteria, and completion signal.

- [x] **Step 2: Create slice plans**

Each slice plan defines exact files, RED tests, focused verification commands, workspace verification gates, status updates, and commit boundaries.

### Task 3: Sync Documentation Navigation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`

- [x] **Step 1: Point active work at M13-01**

Update README and milestone navigation so the current spec and plan are:

```text
docs/specs/M13-01-provider-config.spec.md
docs/plans/M13-01-provider-config.plan.md
```

- [x] **Step 2: Run workspace verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 3: Commit documentation decomposition with M13-01**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 provider config"
```

Expected: commit succeeds with M13 planning docs and the M13-01 implementation.

### Task 4: Implement M13-01 Provider Config

**Files:**
- Read: `docs/specs/M13-01-provider-config.spec.md`
- Execute: `docs/plans/M13-01-provider-config.plan.md`

- [x] **Step 1: Complete M13-01 using TDD**

Follow `docs/plans/M13-01-provider-config.plan.md`: write RED tests, verify failure, implement config contracts, verify focused tests, run workspace gates, update status, and commit.

### Task 5: Implement Remaining M13 Slices In Order

**Files:**
- Execute: `docs/plans/M13-02-provider-registry-capabilities.plan.md`
- Execute: `docs/plans/M13-03-invocation-policy.plan.md`
- Execute: `docs/plans/M13-04-openai-compatible-adapter.plan.md`
- Execute: `docs/plans/M13-05-cli-model-selection.plan.md`

- [x] **Step 1: Implement M13-02 provider registry capabilities**

Complete provider registry and capability negotiation, then commit `feat: add m13 provider registry`.

- [x] **Step 2: Implement M13-03 invocation policy**

Complete timeout, retry, and fallback contracts, then commit `feat: add m13 model invocation policy`.

- [ ] **Step 3: Implement M13-04 OpenAI-compatible adapter**

Complete fixture-backed adapter request and stream parsing, then commit `feat: add m13 openai compatible adapter`.

- [ ] **Step 4: Implement M13-05 CLI model selection**

Complete CLI model selection wiring, then commit `feat: add m13 cli model selection`.

### Task 6: Complete M13

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Mark M13 complete**

Update M13 planning status and advance active focus to the next milestone.

- [ ] **Step 3: Commit M13 completion status**

Run:

```bash
git add docs README.md AGENTS.md
git commit -m "docs: mark m13 real model providers complete"
```

Expected: commit succeeds.
