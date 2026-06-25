# M12-05 Platform Capability Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed platform capability reports in `agent-sandbox`.

**Architecture:** Keep platform detection and sandbox capability reporting in `agent-sandbox`. Use deterministic builders in tests so results do not depend on a specific operating system.

**Tech Stack:** Rust 2024 edition, serde, thiserror, existing `agent-sandbox`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M12-05-platform-capability-report.spec.md`
- Read: `crates/agent-sandbox/src/lib.rs`
- Read: `crates/agent-sandbox/tests/path.rs`

- [ ] **Step 1: Review sandbox ownership**

Run:

```bash
sed -n '1,220p' crates/agent-sandbox/src/lib.rs
sed -n '1,240p' docs/specs/M12-05-platform-capability-report.spec.md
```

Expected: platform reporting belongs in `agent-sandbox`.

### Task 2: Write RED Platform Tests

**Files:**
- Create: `crates/agent-sandbox/tests/platform_report.rs`

- [ ] **Step 1: Add failing tests**

Add tests:

- `platform_report_serializes_stable_shape`
- `platform_report_builder_rejects_empty_os`
- `platform_report_default_includes_current_os`
- `platform_report_represents_unsupported_capabilities`
- `platform_report_includes_plugin_isolation_capabilities`

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-sandbox platform_report
```

Expected: tests fail because platform report contracts do not exist.

### Task 3: Implement Platform Report

**Files:**
- Modify: `crates/agent-sandbox/src/lib.rs`
- Create: `crates/agent-sandbox/src/platform.rs`

- [ ] **Step 1: Add report types**

Implement `PlatformCapabilityReport`, `SandboxCapability`, `PluginIsolationCapability`, `SecretProviderCapability`, and `PlatformReportError`.

- [ ] **Step 2: Add deterministic builder**

Implement a builder that accepts explicit capability values and rejects empty OS names.

- [ ] **Step 3: Add default report**

Implement `PlatformCapabilityReport::current()` using `std::env::consts::OS` and conservative capability defaults.

### Task 4: Verify GREEN

**Files:**
- Read: `crates/agent-sandbox/tests/platform_report.rs`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-sandbox platform_report
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

### Task 5: Update Status And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M12-05-platform-capability-report.plan.md`

- [ ] **Step 1: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m12 platform capability report"
```

Expected: commit succeeds.
