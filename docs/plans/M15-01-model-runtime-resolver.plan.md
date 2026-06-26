# M15-01 Model Runtime Resolver Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve effective runtime config into an executable model provider handle with capability checks.

**Architecture:** Add `agent-core::model_runtime` for config-to-provider orchestration. Keep provider implementations in `agent-model`; keep CLI rendering out of core.

**Tech Stack:** Rust 2024 edition, `Arc<dyn ModelProvider>`, existing model registry contracts, serde-safe errors, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-01-model-runtime-resolver.spec.md`
- Read: `docs/specs/M15-00-config-driven-turn-execution-technical-design.spec.md`
- Read: `crates/agent-core/src/config.rs`
- Read: `crates/agent-model/src/registry.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M15-01-model-runtime-resolver.spec.md
```

Expected: the spec defines resolver contracts, errors, security constraints, and tests.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-core/tests/model_runtime_resolver.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `model_runtime_resolver_selects_effective_provider`
- `model_runtime_resolver_rejects_unregistered_provider`
- `model_runtime_resolver_rejects_disabled_provider`
- `model_runtime_resolver_rejects_missing_capabilities`
- `model_runtime_resolver_error_output_excludes_secret_values`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-core --test model_runtime_resolver
```

Expected: tests fail because `agent_core::model_runtime` does not exist.

### Task 3: Implement Resolver

**Files:**
- Create: `crates/agent-core/src/model_runtime.rs`
- Modify: `crates/agent-core/src/lib.rs`

- [x] **Step 1: Add public contracts**

Implement:

- `ConfiguredModelRuntime`
- `ModelRuntimeResolver`
- `ModelRuntimeError`

- [x] **Step 2: Implement selection logic**

Rules:

- read selected provider/model from `EffectiveRuntimeConfig`
- reject disabled provider configs
- call `ModelProviderRegistry::require_capabilities`
- return provider handle, capabilities, and invocation policy

- [x] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-core --test model_runtime_resolver
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-01-model-runtime-resolver.plan.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

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
git commit -m "feat: add m15 model runtime resolver"
```

Expected: commit succeeds.
