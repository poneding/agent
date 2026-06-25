# M12-06 Benchmarks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic benchmark scenario contracts and smoke tests in `agent-core`.

**Architecture:** Keep benchmark scenario contracts in `agent-core` and use deterministic fake data. Do not add external benchmarking frameworks in M12; this slice proves shape and repeatability.

**Tech Stack:** Rust 2024 edition, serde, std time, existing core/storage/tool/context crates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-06-benchmarks.spec.md`
- Read: `crates/agent-core/src/lib.rs`
- Read: `crates/agent-core/tests/spine_integration.rs`

- [ ] **Step 1: Review deterministic benchmark scope**

Run:

```bash
sed -n '1,240p' docs/specs/M12-06-benchmarks.spec.md
```

Expected: M12 benchmarks are smoke scenarios, not statistical performance claims.

### Task 2: Write RED Benchmark Tests

**Files:**
- Create: `crates/agent-core/tests/benchmarks.rs`

- [ ] **Step 1: Add failing tests**

Add tests:

- `benchmarks_scenario_serializes_stable_shape`
- `benchmarks_runner_rejects_invalid_scenario`
- `benchmarks_runner_records_operation_counts`
- `benchmarks_result_includes_environment_summary`
- `benchmarks_smoke_scenarios_execute_without_external_services`

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core benchmarks
```

Expected: tests fail because benchmark contracts do not exist.

### Task 3: Implement Benchmark Contracts

**Files:**
- Modify: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/src/benchmark.rs`

- [ ] **Step 1: Add benchmark types**

Implement `BenchmarkScenario`, `BenchmarkResult`, `BenchmarkEnvironment`, `BenchmarkRunner`, and `BenchmarkError`.

- [ ] **Step 2: Add deterministic runner**

Implement a runner that validates scenarios, executes a provided deterministic operation, and records elapsed micros and operation count.

- [ ] **Step 3: Add smoke scenarios**

Implement local smoke scenario builders for replay, tool dispatch, event append, and context compaction using fake or in-memory data.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-core/tests/benchmarks.rs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core benchmarks
```

Expected: focused tests pass.

- [ ] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Complete M12 And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-06-benchmarks.plan.md`
- Modify: `docs/plans/M12-production-hardening.plan.md`

- [ ] **Step 1: Mark M12 complete**

Update milestone status and active focus after all M12 acceptance criteria pass.

- [ ] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m12 production hardening"
```

Expected: commit succeeds.
