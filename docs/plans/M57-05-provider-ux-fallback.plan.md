# M57-05 Provider UX Fallback Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed provider support summaries, redacted config-check UX, compatible-only fallback selection, and deterministic M56 coding-path fallback coverage.

**Architecture:** Capability matching remains in `ruse-model`; provider support reporting and runtime fallback decisions live in `ruse-core`; CLI and UI bridge render typed support summaries without owning provider rules.

**Tech Stack:** Rust 2024 edition, serde contracts, async fixture providers, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M57-05-provider-ux-fallback.spec.md`
- Read: `docs/specs/M57-multi-provider-runtime.spec.md`
- Read: `crates/ruse-model/src/registry.rs`
- Read: `crates/ruse-model/src/invocation_policy.rs`
- Read: `crates/ruse-core/src/model_runtime.rs`
- Read: `crates/ruse-core/tests/model_invocation_policy_runner.rs`
- Read: `crates/ruse-core/tests/streaming_retry_cancel_replay.rs`
- Read: `crates/ruse-core/tests/comprehensive_local_coding_e2e.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-cli/tests/config_diagnostics.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review current provider support and fallback baseline**

Run:

```bash
cargo test -p ruse-model invocation_policy
cargo test -p ruse-core model_invocation_policy_runner
cargo test -p ruse-core streaming_retry_cancel_replay
cargo test -p ruse-core comprehensive_local_coding_e2e
cargo test -p ruse-cli config_diagnostics
```

Expected: existing fallback, streaming retry, local coding, and config diagnostics pass before M57-05 changes.

### Task 2: Write Failing Tests

**Files:**
- Modify: `crates/ruse-model/tests/invocation_policy.rs`
- Modify: `crates/ruse-core/tests/model_runtime_resolver.rs`
- Modify: `crates/ruse-core/tests/model_invocation_policy_runner.rs`
- Modify: `crates/ruse-core/tests/streaming_retry_cancel_replay.rs`
- Modify: `crates/ruse-core/tests/comprehensive_local_coding_e2e.rs`
- Modify: `crates/ruse-cli/tests/config_diagnostics.rs`
- Modify if needed: `crates/ruse-ui-bridge/tests/*`

- [x] **Step 1: Add provider compatibility tests**

Cover:

- selected provider compatibility includes stable missing capability labels
- fallback support report preserves configured fallback order
- disabled providers are reported as incompatible
- support summaries do not include endpoint URLs, secret ref names, prompt text, or response body text

- [x] **Step 2: Add runtime fallback tests**

Cover:

- selected provider missing required capabilities can resolve to the first compatible fallback before invocation
- fallback selection skips registered providers missing required capabilities and invokes the next compatible provider
- missing fallback provider ids remain terminal configuration errors
- no fallback occurs after a stream crosses the side-effect boundary with text or tool-call output

- [x] **Step 3: Add CLI/JSON support-output tests**

Cover:

- text config check includes a provider support summary for selected and fallback providers
- JSON config snapshot exposes typed provider support summaries
- support output reports missing capabilities as labels
- local endpoint URLs, API key provider/name pairs, raw secret-like values, and query strings are not present

- [x] **Step 4: Add M56 fallback regression test**

Cover:

- deterministic local coding fixture completes when the selected provider is incompatible or fails safely and a tool-capable fallback provider satisfies the coding requirements
- final output and attempt/provider evidence stay stable

- [x] **Step 5: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-model invocation_policy
cargo test -p ruse-core model_runtime_resolver
cargo test -p ruse-core model_invocation_policy_runner
cargo test -p ruse-core streaming_retry_cancel_replay
cargo test -p ruse-core comprehensive_local_coding_e2e
cargo test -p ruse-cli config_diagnostics
```

Expected: new tests fail before provider support reports, JSON contracts, and compatible-only fallback behavior are implemented.

### Task 3: Implement Provider Support Contracts

**Files:**
- Modify: `crates/ruse-model/src/registry.rs`
- Modify if needed: `crates/ruse-model/src/invocation_policy.rs`
- Create: `crates/ruse-core/src/provider_support.rs`
- Modify: `crates/ruse-core/src/lib.rs`
- Modify: `crates/ruse-core/src/model_runtime.rs`

- [x] **Step 1: Add coding-turn requirements helper**

Add a shared helper for M56/M57 coding-agent model requirements so config-check and runtime tests evaluate the same capability profile.

- [x] **Step 2: Add typed provider support report**

Add serde-friendly provider support summary types that include provider id, kind, role, fallback index, enabled state, compatibility, and missing capabilities.

- [x] **Step 3: Build reports from effective config**

Construct support reports from `EffectiveRuntimeConfig`, default provider capabilities, invocation fallback order, and the shared coding requirements.

- [x] **Step 4: Keep diagnostics redacted and bounded**

Ensure support report construction never includes endpoint URLs, secret refs, prompts, response bodies, or raw transport errors.

### Task 4: Implement Compatible-Only Fallback

**Files:**
- Modify: `crates/ruse-core/src/model_runtime.rs`
- Modify if needed: `crates/ruse-model/src/invocation_policy.rs`

- [x] **Step 1: Resolve selected-provider incompatibility through fallback**

When the selected provider misses requirements and a compatible fallback exists, resolve the runtime to the first compatible fallback without invoking the incompatible provider.

- [x] **Step 2: Skip incompatible fallback candidates after failures**

During retry/fallback execution, skip fallback providers that are registered but missing capabilities and continue to the next fallback candidate.

- [x] **Step 3: Preserve terminal diagnostics**

If no compatible fallback exists, return stable terminal errors that include provider ids and missing capability labels, without prompts or secrets.

- [x] **Step 4: Preserve side-effect boundary behavior**

Do not retry or fallback after a stream has emitted text or tool-call output.

### Task 5: Wire CLI and UI Bridge Support Output

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify if needed: `crates/ruse-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Add text support summary**

Add a stable `Provider support:` line to `--config-check` output that names selected/fallback compatibility and missing capabilities.

- [x] **Step 2: Add JSON support contracts**

Extend config JSON output additively with provider support summaries, either on `UiConfigSnapshot` or as a dedicated frame.

- [x] **Step 3: Redact support output**

Verify output does not expose endpoint URLs, credentials, secret ref names, prompt text, query strings, or provider response bodies.

### Task 6: Verify and Commit

**Files:**
- Modify: `docs/plans/M57-05-provider-ux-fallback.plan.md`
- Modify: `docs/plans/M57-multi-provider-runtime.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-model -p ruse-core -p ruse-cli -p ruse-ui-bridge --all-targets -- -D warnings
cargo test -p ruse-model invocation_policy
cargo test -p ruse-core model_runtime_resolver
cargo test -p ruse-core model_invocation_policy_runner
cargo test -p ruse-core streaming_retry_cancel_replay
cargo test -p ruse-core comprehensive_local_coding_e2e
cargo test -p ruse-cli config_diagnostics
```

Expected: focused checks pass.

- [x] **Step 2: Commit implementation**

Run exact-path staging, then:

```bash
git commit -m "feat: add m57 provider support fallback"
```

Expected: commit contains provider support contracts, compatible-only fallback behavior, CLI/JSON support output, tests, and docs/status updates.

### Completion Checklist

- [x] Provider support summaries are typed, serde-friendly, and redacted.
- [x] Config-check text and JSON expose selected/fallback compatibility.
- [x] Runtime fallback skips incompatible providers before invocation.
- [x] M56 local coding path has deterministic compatible-fallback coverage.
- [x] Focused fmt/clippy/tests pass.
