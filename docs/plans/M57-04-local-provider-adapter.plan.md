# M57-04 Local Provider Adapter Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a local/Ollama-compatible provider adapter with local endpoint safety, fixture-backed streaming/tool-call mapping, typed capabilities, CLI registration, and redacted diagnostics.

**Architecture:** Local provider protocol and endpoint safety live in `ruse-model`; `ruse-cli` only builds configured providers and renders typed config summaries.

**Tech Stack:** Rust 2024 edition, async trait providers, serde JSONL fixtures, reqwest transport, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M57-04-local-provider-adapter.spec.md`
- Read: `crates/ruse-model/src/gemini.rs`
- Read: `crates/ruse-model/tests/gemini.rs`
- Read: `crates/ruse-model/src/config.rs`
- Read: `crates/ruse-model/src/registry.rs`
- Read: `crates/ruse-cli/src/providers.rs`
- Read: `crates/ruse-cli/tests/live_provider_registration.rs`

- [x] **Step 1: Review current provider adapter baseline**

Run:

```bash
cargo test -p ruse-model gemini
cargo test -p ruse-cli live_provider_registration
```

Expected: Gemini adapter behavior and CLI provider registration pass before adding local provider support.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/ruse-model/tests/local_provider.rs`
- Modify: `crates/ruse-model/tests/provider_config.rs`
- Modify: `crates/ruse-model/tests/provider_capability_matrix.rs`
- Modify: `crates/ruse-cli/tests/config_diagnostics.rs`
- Modify: `crates/ruse-cli/tests/live_provider_registration.rs`

- [x] **Step 1: Add ruse-model local provider tests**

Cover:

- request body shape for local/Ollama chat
- endpoint URL builder appends `/api/chat`
- provider requires a local base URL but not an API key
- invalid provider kind is rejected
- text delta and completion JSONL mapping
- tool-call mapping to `ModelStreamEvent::ToolCall`
- provider response and transport error redaction/retryability

- [x] **Step 2: Add config and capability tests**

Cover:

- `ModelProviderKind::Local` serde shape
- local providers do not require a secret ref
- local providers require explicit local `base_url`
- remote HTTP/HTTPS endpoints are rejected for local providers
- default local capability profile includes streaming and tool calls

- [x] **Step 3: Add CLI diagnostics and registration tests**

Cover:

- text config check includes local capability summary
- JSON config snapshot includes local capability metadata
- enabled local provider registers without a secret resolver entry
- local provider summaries do not expose endpoint credentials or query strings

- [x] **Step 4: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-model local_provider
cargo test -p ruse-model provider_config
cargo test -p ruse-model provider_capability_matrix
cargo test -p ruse-cli config_diagnostics
cargo test -p ruse-cli live_provider_registration
```

Expected: tests fail before local provider contracts and implementation exist.

### Task 3: Implement Local Model Adapter

**Files:**
- Create: `crates/ruse-model/src/local.rs`
- Modify: `crates/ruse-model/src/lib.rs`
- Modify: `crates/ruse-model/src/config.rs`
- Modify: `crates/ruse-model/src/registry.rs`

- [x] **Step 1: Extend typed provider kind and endpoint validation**

Add `ModelProviderKind::Local`, require explicit local `base_url`, allow loopback `http`/`https`, and reject non-local endpoints.

- [x] **Step 2: Add request builder and transport contracts**

Add `LocalHttpRequest`, `LocalHttpResponse`, `LocalTransport`, fixture transport, and reqwest transport with JSON body, timeout, status handling, and URL/message redaction.

- [x] **Step 3: Add provider and JSONL stream parser**

Implement `LocalProvider`, text delta parsing, Ollama-style tool-call mapping, completion mapping, unknown-field tolerance, bounded error messages, and retryability classification.

- [x] **Step 4: Add default capabilities**

Add `ModelCapabilities::local_default()` and update `default_for_provider_kind`.

### Task 4: Wire CLI and Config Surfaces

**Files:**
- Modify: `crates/ruse-cli/src/providers.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify if needed: `crates/ruse-core/src/repo_session.rs`
- Modify if needed: `crates/ruse-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Register configured local providers**

Instantiate `LocalProvider` for enabled local configs without requiring a secret.

- [x] **Step 2: Render local kind/capabilities**

Update provider kind labels and ensure existing config check text/JSON summaries include local capability metadata without exposing endpoint URLs, credentials, or query strings.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M57-04-local-provider-adapter.plan.md`
- Modify: `docs/milestones/README.md`
- Modify if needed: `docs/plans/M57-multi-provider-runtime.plan.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-model -p ruse-core -p ruse-cli -p ruse-ui-bridge --all-targets -- -D warnings
cargo test -p ruse-model local_provider
cargo test -p ruse-model provider_config
cargo test -p ruse-model provider_capability_matrix
cargo test -p ruse-cli config_diagnostics
cargo test -p ruse-cli live_provider_registration
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging, then:

```bash
git commit -m "feat: add m57 local provider adapter"
```

Expected: commit contains local provider contracts, adapter implementation, CLI wiring, tests, and docs/status updates.

### Completion Checklist

- [x] Local provider kind and endpoint validation are typed and serde-tested.
- [x] Local request builder and JSONL stream parser are fixture-tested.
- [x] Tool-call streaming maps to `ModelStreamEvent::ToolCall`.
- [x] Provider errors are bounded, retryable where appropriate, and redacted.
- [x] CLI registration and config diagnostics include local providers without secrets.
- [x] Focused fmt/clippy/tests pass.
