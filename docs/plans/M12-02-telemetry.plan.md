# M12-02 Telemetry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a provider-neutral `ruse-telemetry` crate with validated spans, metrics, trace context, redaction, and in-memory sink tests.

**Architecture:** Keep telemetry contracts below `ruse-core`; telemetry data may use `ruse-types` ids but must not depend on runtime orchestration or UI crates.

**Tech Stack:** Rust 2024 edition, serde, serde_json, thiserror, Cargo workspace, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-02-telemetry.spec.md`
- Read: `docs/specs/M12-00-production-hardening-technical-design.spec.md`
- Read: `Cargo.toml`

- [x] **Step 1: Review telemetry crate boundary**

Run:

```bash
sed -n '1,260p' docs/specs/M12-02-telemetry.spec.md
```

Expected: telemetry is provider-neutral and lower-level than `ruse-core`.

### Task 2: Write RED Telemetry Tests

**Files:**
- Create: `crates/ruse-telemetry/tests/telemetry.rs`
- Create: `crates/ruse-telemetry/Cargo.toml`
- Create: `crates/ruse-telemetry/src/lib.rs`

- [x] **Step 1: Add failing tests**

Add tests:

- `telemetry_span_serializes_stable_shape`
- `telemetry_metric_rejects_non_finite_value`
- `telemetry_sink_records_spans_and_metrics_in_order`
- `telemetry_redacts_secret_like_attributes`
- `telemetry_trace_context_rejects_empty_identifiers`

The tests define the public API for `TraceContext`, `TelemetrySpan`, `TelemetryMetric`, `TelemetryAttribute`, `TelemetryError`, `TelemetryRecord`, `TelemetrySink`, and `InMemoryTelemetrySink`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-telemetry telemetry
```

Expected: tests fail because telemetry contracts are not implemented.

### Task 3: Implement Telemetry Crate

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/ruse-telemetry/Cargo.toml`
- Modify: `crates/ruse-telemetry/src/lib.rs`
- Create: `crates/ruse-telemetry/src/span.rs`
- Create: `crates/ruse-telemetry/src/metric.rs`
- Create: `crates/ruse-telemetry/src/sink.rs`

- [x] **Step 1: Register workspace crate**

Add `crates/ruse-telemetry` to workspace members and define dependencies on `ruse-types`, `serde`, `serde_json`, and `thiserror`.

- [x] **Step 2: Implement span and trace context types**

Implement `TraceContext`, `TelemetrySpan`, `TelemetryAttribute`, and validation constructors.

- [x] **Step 3: Implement metric types**

Implement `TelemetryMetric` with finite numeric validation and explicit unit validation.

- [x] **Step 4: Implement sink and redaction**

Implement `TelemetrySink`, `InMemoryTelemetrySink`, and redaction for secret-like attribute keys.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/ruse-telemetry/tests/telemetry.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p ruse-telemetry telemetry
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-02-telemetry.plan.md`

- [x] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m12 telemetry"
```

Expected: commit succeeds.
