# M17-03 Live Provider Registration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build CLI provider registries from effective config, including live OpenAI-compatible providers.

**Architecture:** `agent-cli` owns process-level provider registration. `agent-core` still receives a provider-neutral `ModelProviderRegistry`.

**Tech Stack:** Rust 2024 edition, existing config loader, `agent-model` providers, env secret resolver, fixture provider factories, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Registration Tests

**Files:**
- Create: `crates/agent-cli/tests/live_provider_registration.rs`
- Modify: `crates/agent-cli/tests/prompt_runtime.rs`

- [x] **Step 1: Add fake and live registration test**

Add `live_provider_registration_registers_enabled_config_providers`.

Use a config with enabled fake and OpenAI-compatible providers.

- [x] **Step 2: Add disabled provider test**

Add `live_provider_registration_skips_disabled_providers`.

- [x] **Step 3: Add missing secret test**

Add `live_provider_registration_missing_env_secret_fails_before_network`.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-cli --test live_provider_registration
cargo test -p agent-cli --test prompt_runtime
```

Expected: failures because CLI registry builder only constructs fake providers.

### Task 2: Extract Registry Builder

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Create: `crates/agent-cli/src/providers.rs`
- Modify: `crates/agent-cli/src/lib.rs`

- [x] **Step 1: Move fake registry code**

Extract current `fake_provider_registry` behavior into `providers.rs`.

- [x] **Step 2: Add `build_model_provider_registry`**

Input: `EffectiveRuntimeConfig`, safe prompt text, secret resolver factory.

- [x] **Step 3: Preserve fake defaults**

Existing fake prompt runtime tests must remain unchanged.

### Task 3: Register OpenAI-Compatible Providers

**Files:**
- Modify: `crates/agent-cli/src/providers.rs`
- Modify: `crates/agent-cli/tests/live_provider_registration.rs`

- [x] **Step 1: Construct reqwest-backed providers**

For `ModelProviderKind::OpenAiCompatible`, build `OpenAiCompatibleProvider` with `ReqwestOpenAiCompatibleTransport`.

- [x] **Step 2: Resolve missing secret diagnostics before invocation**

Validate required secret refs enough to fail before network attempts.

- [x] **Step 3: Keep errors redacted**

Tests assert no raw env secret value appears in output.

### Task 4: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-cli --test live_provider_registration
cargo test -p agent-cli --test prompt_runtime
cargo test -p agent-core --test model_runtime_resolver
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

Run:

```bash
git add crates/agent-cli docs
git commit -m "feat: register live model providers from config"
```
