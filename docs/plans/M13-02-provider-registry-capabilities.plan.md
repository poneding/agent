# M13-02 Provider Registry Capabilities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add model provider registry and capability negotiation contracts.

**Architecture:** Implement registry contracts inside `ruse-model`. Keep provider execution behind the existing `ModelProvider` trait and avoid adding runtime orchestration to this slice.

**Tech Stack:** Rust 2024 edition, async-trait, serde, thiserror, existing fake provider tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M13-02-provider-registry-capabilities.spec.md`
- Read: `crates/ruse-model/src/provider.rs`
- Read: `crates/ruse-model/src/config.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M13-02-provider-registry-capabilities.spec.md
```

Expected: the spec defines registry and capability behavior.

### Task 2: Write RED Registry Tests

**Files:**
- Create: `crates/ruse-model/tests/provider_registry.rs`

- [x] **Step 1: Add focused tests**

Add tests:

- `provider_registry_registers_and_resolves_provider`
- `provider_registry_rejects_duplicate_provider_ids`
- `provider_registry_reports_provider_not_found`
- `provider_registry_accepts_compatible_capabilities`
- `provider_registry_reports_missing_capabilities`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-model provider_registry
```

Expected: tests fail because registry contracts do not exist.

### Task 3: Implement Registry Contracts

**Files:**
- Create: `crates/ruse-model/src/registry.rs`
- Modify: `crates/ruse-model/src/lib.rs`

- [x] **Step 1: Add capability types**

Implement `ModelCapabilities` and `ModelRequestRequirements` with stable serde shapes.

- [x] **Step 2: Add registry**

Implement `ModelProviderRegistry`, provider registration, provider lookup, duplicate detection, and capability negotiation.

- [x] **Step 3: Export registry types**

Export registry and capability contracts from `ruse-model`.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-02-provider-registry-capabilities.plan.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo test -p ruse-model provider_registry
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 provider registry"
```

Expected: commit succeeds.
