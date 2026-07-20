# M14-03 Config Layering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compose ordered runtime config layers into one effective model config with explicit overrides.

**Architecture:** Extend `ruse-core::config` with pure layering functions. The implementation consumes parsed configs and does not read files, resolve secrets, or invoke models.

**Tech Stack:** Rust 2024 edition, serde contracts, thiserror, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M14-03-config-layering.spec.md`
- Read: `crates/ruse-core/src/config.rs`
- Read: `crates/ruse-core/tests/runtime_config_loader.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,240p' docs/specs/M14-03-config-layering.spec.md
```

Expected: the spec defines layer precedence, overrides, diagnostics, and tests.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/ruse-core/tests/runtime_config_layering.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `config_layering_higher_priority_default_provider_wins`
- `config_layering_merges_provider_entries_by_id`
- `config_layering_explicit_provider_and_model_override_win`
- `config_layering_rejects_disabled_selected_provider`
- `config_layering_summary_serializes_without_secret_values`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-core --test runtime_config_layering
```

Expected: tests fail because layering contracts do not exist.

### Task 3: Implement Layering

**Files:**
- Modify: `crates/ruse-core/src/config.rs`

- [x] **Step 1: Add layer contracts**

Implement:

- `ConfigLayerKind`
- `ConfigLayer`
- `ConfigLayerSummary`
- `ConfigOverride`
- `EffectiveRuntimeConfig`
- `build_effective_runtime_config`

- [x] **Step 2: Implement deterministic merge**

Rules:

- process layers in precedence order
- merge providers by id
- use highest-priority `default_provider`
- apply explicit provider/model overrides last
- reject disabled or missing selected providers

- [x] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p ruse-core --test runtime_config_layering
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-03-config-layering.plan.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m14 config layering"
```

Expected: commit succeeds.
