# M57-02 Anthropic Adapter Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a native Anthropic provider adapter with fixture-backed streaming/tool-use mapping, typed capabilities, CLI registration, and redacted diagnostics.

**Architecture:** Anthropic protocol logic lives in `agent-model`; `agent-cli` only builds configured providers and renders existing typed config summaries.

**Tech Stack:** Rust 2024 edition, async trait providers, serde JSON fixtures, reqwest transport, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M57-02-anthropic-adapter.spec.md`
- Read: `crates/agent-model/src/openai_compatible.rs`
- Read: `crates/agent-model/tests/openai_compatible.rs`
- Read: `crates/agent-model/src/config.rs`
- Read: `crates/agent-model/src/registry.rs`
- Read: `crates/agent-cli/src/providers.rs`
- Read: `crates/agent-cli/tests/live_provider_registration.rs`

- [x] **Step 1: Review current provider adapter baseline**

Run:

```bash
cargo test -p agent-model openai_compatible
cargo test -p agent-cli live_provider_registration
```

Expected: existing OpenAI-compatible request/stream mapping and CLI provider registration pass before adding Anthropic.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-model/tests/anthropic.rs`
- Modify: `crates/agent-model/tests/provider_config.rs`
- Modify: `crates/agent-model/tests/provider_capability_matrix.rs`
- Modify: `crates/agent-cli/tests/config_diagnostics.rs`
- Modify: `crates/agent-cli/tests/live_provider_registration.rs`

- [x] **Step 1: Add agent-model Anthropic tests**

Cover:

- request body shape for the Anthropic Messages API
- provider requires an API-key secret ref
- invalid provider kind is rejected
- text delta and completion SSE mapping
- tool-use stream mapping from partial JSON to `ModelStreamEvent::ToolCall`
- provider response and transport error redaction/retryability

- [x] **Step 2: Add config and capability tests**

Cover:

- `ModelProviderKind::Anthropic` serde shape
- Anthropic providers require a secret ref
- default Anthropic capability profile includes streaming and tool calls
- capability defaults remain deterministic for fake/OpenAI-compatible providers

- [x] **Step 3: Add CLI diagnostics and registration tests**

Cover:

- text config check includes Anthropic capability summary
- JSON config snapshot includes Anthropic capability metadata
- enabled Anthropic provider registers with an in-memory secret
- missing Anthropic env secret fails before network and redacts secret-like values

- [x] **Step 4: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-model anthropic
cargo test -p agent-model provider_config
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-cli config_diagnostics
cargo test -p agent-cli live_provider_registration
```

Expected: tests fail before Anthropic contracts and implementation exist.

### Task 3: Implement Anthropic Model Adapter

**Files:**
- Create: `crates/agent-model/src/anthropic.rs`
- Modify: `crates/agent-model/src/lib.rs`
- Modify: `crates/agent-model/src/config.rs`
- Modify: `crates/agent-model/src/registry.rs`

- [x] **Step 1: Extend typed provider kind and config validation**

Add `ModelProviderKind::Anthropic`, require `api_key`, and keep serde names stable and snake_case.

- [x] **Step 2: Add request builder and transport contracts**

Add `AnthropicHttpRequest`, `AnthropicHttpResponse`, `AnthropicTransport`, fixture transport, and reqwest transport with `x-api-key`, `anthropic-version`, JSON body, timeout, status handling, and URL/message redaction.

- [x] **Step 3: Add provider and stream parser**

Implement `AnthropicProvider`, text delta parsing, tool-use accumulation by content-block index, completion mapping, unknown-event tolerance, bounded error messages, and retryability classification.

- [x] **Step 4: Add default capabilities**

Add `ModelCapabilities::anthropic_default()` and update `default_for_provider_kind`.

### Task 4: Wire CLI and Config Surfaces

**Files:**
- Modify: `crates/agent-cli/src/providers.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Register configured Anthropic providers**

Instantiate `AnthropicProvider` for enabled Anthropic configs, resolve secrets before live invocation, and redact registration errors.

- [x] **Step 2: Render Anthropic kind/capabilities**

Update provider kind labels and ensure existing config check text/JSON summaries include Anthropic capability metadata without exposing endpoint URLs or secret refs.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M57-02-anthropic-adapter.plan.md`
- Modify: `docs/milestones/README.md`
- Modify if needed: `docs/plans/M57-multi-provider-runtime.plan.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-model -p agent-cli -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-model anthropic
cargo test -p agent-model provider_config
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-cli config_diagnostics
cargo test -p agent-cli live_provider_registration
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging, then:

```bash
git commit -m "feat: add m57 anthropic provider adapter"
```

Expected: commit contains Anthropic contracts, adapter implementation, CLI wiring, tests, and docs/status updates.

### Completion Checklist

- [x] Anthropic provider kind and config validation are typed and serde-tested.
- [x] Anthropic request builder and stream parser are fixture-tested.
- [x] Tool-use streaming maps to `ModelStreamEvent::ToolCall`.
- [x] Provider errors are bounded, retryable where appropriate, and redacted.
- [x] CLI registration and config diagnostics include Anthropic without secrets.
- [x] Focused fmt/clippy/tests pass.
