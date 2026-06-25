# M12 Production Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the runtime reliable for daily use and external integration with migrations, telemetry, audit export, memory portability, platform reports, and benchmarks.

**Architecture:** Implement this milestone behind the crate boundaries named in the matching spec. Keep shared contracts in lower-level crates, route orchestration through `agent-core`, and expose user-facing behavior through CLI or UI bridge crates only when the spec calls for it.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde-compatible contracts where durable or wire formats are involved, Tokio for async runtime paths that need it, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Milestone Inputs

**Files:**
- Read: `docs/specs/M12-production-hardening.spec.md`
- Read: `docs/milestones/README.md`
- Read: `docs/PRD.md`

- [ ] **Step 1: Review the milestone spec**

Run:

```bash
sed -n '1,260p' docs/specs/M12-production-hardening.spec.md
```

Expected: the spec describes purpose, scope, contracts, tests, and acceptance criteria for M12.

- [ ] **Step 2: Confirm workspace health before changes**

Run:

```bash
cargo test --workspace
```

Expected: existing workspace tests pass before this milestone begins.

### Task 2: Add Contract Tests First

**Files:**
- Create/Modify: `crates/agent-core/tests/m12_production_hardening.rs`

- [ ] **Step 1: Write tests for milestone behavior**

Cover these behaviors:

- migration fixture tests
- telemetry field tests
- audit export tests
- memory archive tests
- platform report tests
- benchmark smoke tests

Create focused tests with names that describe the contract being proven.

Recommended test names:

- `m12_production_hardening_validates_public_contracts`
- `m12_production_hardening_rejects_invalid_inputs`
- `m12_production_hardening_emits_expected_runtime_events`
- `m12_production_hardening_preserves_serialized_shape`
- `m12_production_hardening_reports_recoverable_errors`

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test --workspace m12_production_hardening
```

Expected: tests fail because the milestone contracts are not implemented yet.

### Task 3: Implement Milestone Contracts

**Files:**
- Modify/Create: `crates/agent-storage/src/lib.rs`
- Modify/Create: `crates/agent-telemetry/src/lib.rs`
- Modify/Create: `crates/agent-memory/src/lib.rs`
- Modify/Create: `crates/agent-sandbox/src/lib.rs`
- Modify/Create: `crates/agent-core/src/lib.rs`
- Modify/Create: `crates/agent-cli/src/lib.rs`

- [ ] **Step 1: Add or extend crate modules**

Implement the public contracts listed in the spec:

- SchemaMigration contract
- TelemetrySpan contract
- AuditExport contract
- MemoryArchive contract
- PlatformCapabilityReport contract
- BenchmarkScenario contract

- [ ] **Step 2: Keep dependency direction clean**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and newly touched crates do not introduce reverse dependencies into lower-level crates.

- [ ] **Step 3: Run focused tests and verify success**

Run:

```bash
cargo test --workspace m12_production_hardening
```

Expected: focused milestone tests pass.

### Task 4: Add Integration Coverage

**Files:**
- Create/Modify: `crates/agent-core/tests/m12_production_hardening_integration.rs`

- [ ] **Step 1: Add integration assertions for acceptance criteria**

Acceptance criteria to encode:

- Users can upgrade data safely.
- Runtime behavior is observable in logs, metrics, traces, and audit exports.
- Core benchmarks are tracked and repeatable.

- [ ] **Step 2: Run integration tests**

Run:

```bash
cargo test --workspace m12_production_hardening_integration
```

Expected: integration tests pass and fail deterministically when the milestone behavior is removed.

### Task 5: Run Full Verification

**Files:**
- Read: `Cargo.toml`
- Read: `Cargo.lock`

- [ ] **Step 1: Format check**

Run:

```bash
cargo fmt --all -- --check
```

Expected: formatting check passes.

- [ ] **Step 2: Lint check**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: clippy passes without warnings.

- [ ] **Step 3: Full test suite**

Run:

```bash
cargo test --workspace
```

Expected: all workspace tests pass.

### Task 6: Update Progress and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [ ] **Step 1: Update active milestone status**

Update `docs/milestones/README.md` so the completed milestone is marked as complete or the active focus advances to the next milestone.

- [ ] **Step 2: Commit the milestone**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m12 production hardening"
```

Expected: commit succeeds with source, tests, and documentation staged together.
