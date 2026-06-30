# M57-03 Gemini Adapter Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native Gemini provider adapter with fixture-backed streaming/function-call mapping, typed capabilities, CLI registration, and redacted diagnostics.

**Architecture:** Gemini protocol logic lives in `agent-model`; `agent-cli` only builds configured providers and renders typed config summaries.

**Tech Stack:** Rust 2024 edition, async trait providers, serde JSON fixtures, reqwest transport, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M57-03-gemini-adapter.spec.md`
- Read: `crates/agent-model/src/anthropic.rs`
- Read: `crates/agent-model/tests/anthropic.rs`
- Read: `crates/agent-model/src/config.rs`
- Read: `crates/agent-model/src/registry.rs`
- Read: `crates/agent-cli/src/providers.rs`
- Read: `crates/agent-cli/tests/live_provider_registration.rs`

- [ ] **Step 1: Review current native provider adapter baseline**

Run:

```bash
cargo test -p agent-model anthropic
cargo test -p agent-cli live_provider_registration
```

Expected: Anthropic adapter behavior and CLI provider registration pass before adding Gemini.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-model/tests/gemini.rs`
- Modify: `crates/agent-model/tests/provider_config.rs`
- Modify: `crates/agent-model/tests/provider_capability_matrix.rs`
- Modify: `crates/agent-cli/tests/config_diagnostics.rs`
- Modify: `crates/agent-cli/tests/live_provider_registration.rs`

- [ ] **Step 1: Add agent-model Gemini tests**

Cover:

- request body shape for Gemini `streamGenerateContent`
- URL builder keeps API key out of the durable URL
- provider requires an API-key secret ref
- invalid provider kind is rejected
- text delta and completion SSE mapping
- function-call mapping to `ModelStreamEvent::ToolCall`
- provider response and transport error redaction/retryability

- [ ] **Step 2: Add config and capability tests**

Cover:

- `ModelProviderKind::Gemini` serde shape
- Gemini providers require a secret ref
- default Gemini capability profile includes streaming and tool calls
- existing provider defaults remain deterministic

- [ ] **Step 3: Add CLI diagnostics and registration tests**

Cover:

- text config check includes Gemini capability summary
- JSON config snapshot includes Gemini capability metadata
- enabled Gemini provider registers with an in-memory secret
- missing Gemini env secret fails before network and redacts secret-like values

- [ ] **Step 4: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-model gemini
cargo test -p agent-model provider_config
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-cli config_diagnostics
cargo test -p agent-cli live_provider_registration
```

Expected: tests fail before Gemini contracts and implementation exist.

### Task 3: Implement Gemini Model Adapter

**Files:**
- Create: `crates/agent-model/src/gemini.rs`
- Modify: `crates/agent-model/src/lib.rs`
- Modify: `crates/agent-model/src/config.rs`
- Modify: `crates/agent-model/src/registry.rs`

- [ ] **Step 1: Extend typed provider kind and config validation**

Add `ModelProviderKind::Gemini`, require `api_key`, and keep serde names stable and snake_case.

- [ ] **Step 2: Add request builder and transport contracts**

Add `GeminiHttpRequest`, `GeminiHttpResponse`, `GeminiTransport`, fixture transport, and reqwest transport with `x-goog-api-key`, JSON body, timeout, status handling, and URL/message redaction.

- [ ] **Step 3: Add provider and stream parser**

Implement `GeminiProvider`, text delta parsing, function-call mapping, completion mapping, unknown-field tolerance, bounded error messages, and retryability classification.

- [ ] **Step 4: Add default capabilities**

Add `ModelCapabilities::gemini_default()` and update `default_for_provider_kind`.

### Task 4: Wire CLI and Config Surfaces

**Files:**
- Modify: `crates/agent-cli/src/providers.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-core/src/repo_session.rs`
- Modify if needed: `crates/agent-cli/tests/config_diagnostics.rs`

- [ ] **Step 1: Register configured Gemini providers**

Instantiate `GeminiProvider` for enabled Gemini configs, resolve secrets before live invocation, and redact registration errors.

- [ ] **Step 2: Render Gemini kind/capabilities**

Update provider kind labels and ensure existing config check text/JSON summaries include Gemini capability metadata without exposing endpoint URLs or secret refs.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M57-03-gemini-adapter.plan.md`
- Modify: `docs/milestones/README.md`
- Modify if needed: `docs/plans/M57-multi-provider-runtime.plan.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-model -p agent-cli -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-model gemini
cargo test -p agent-model provider_config
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-cli config_diagnostics
cargo test -p agent-cli live_provider_registration
```

Expected: focused checks pass.

- [ ] **Step 2: Commit**

Run exact-path staging, then:

```bash
git commit -m "feat: add m57 gemini provider adapter"
```

Expected: commit contains Gemini contracts, adapter implementation, CLI wiring, tests, and docs/status updates.

### Completion Checklist

- [ ] Gemini provider kind and config validation are typed and serde-tested.
- [ ] Gemini request builder and stream parser are fixture-tested.
- [ ] Function-call streaming maps to `ModelStreamEvent::ToolCall`.
- [ ] Provider errors are bounded, retryable where appropriate, and redacted.
- [ ] CLI registration and config diagnostics include Gemini without secrets.
- [ ] Focused fmt/clippy/tests pass.
