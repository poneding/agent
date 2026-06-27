# M17-02 OpenAI Reqwest Transport Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real reqwest-backed OpenAI-compatible streaming transport with offline tests.

**Architecture:** `agent-model::openai_compatible` owns reqwest transport and SSE parsing. Tests use fixture/local HTTP streams; no test calls external services.

**Tech Stack:** Rust 2024 edition, `reqwest` with rustls/json/stream, Tokio, futures stream utilities, serde JSON, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Transport Tests

**Files:**
- Modify: `crates/agent-model/tests/openai_compatible.rs`
- Create: `crates/agent-model/tests/openai_reqwest_transport.rs`

- [x] **Step 1: Add request construction test**

Add `reqwest_transport_builds_responses_request`.

Assert URL ends with `/responses`, body has `stream: true`, and timeout comes from config.

- [x] **Step 2: Add secret header redaction test**

Add `reqwest_transport_injects_secret_without_debug_leak`.

Use `InMemorySecretResolver`; assert request debug/error strings do not contain raw secret values.

- [x] **Step 3: Add fixture streaming test**

Add `reqwest_transport_streams_sse_chunks_into_model_events`.

Use an in-process fixture transport or local HTTP server; assert deltas are collected in order.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-model --test openai_reqwest_transport
cargo test -p agent-model --test openai_compatible
```

Expected: failures because reqwest transport is not implemented.

### Task 2: Add Reqwest Transport

**Files:**
- Modify: `crates/agent-model/Cargo.toml`
- Modify: `crates/agent-model/src/openai_compatible.rs`
- Modify: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Add reqwest dependency**

Use `default-features = false` with `rustls-tls`, `json`, and `stream`.

- [x] **Step 2: Implement `ReqwestOpenAiCompatibleTransport`**

Constructor receives `Arc<dyn SecretResolver>` and an optional `reqwest::Client` for tests.

- [x] **Step 3: Resolve secrets at send time**

Read `SecretRef` immediately before request execution and inject `Authorization: Bearer ...`.

- [x] **Step 4: Stream response bytes**

Convert response bytes into SSE frame strings and then provider-neutral model stream events.

### Task 3: Classify And Redact Errors

**Files:**
- Modify: `crates/agent-model/src/openai_compatible.rs`
- Modify: `crates/agent-model/tests/openai_reqwest_transport.rs`

- [x] **Step 1: Map HTTP statuses**

Classify 400/401/403 as terminal and 408/429/5xx as retryable.

- [x] **Step 2: Limit and redact response body summaries**

Preserve existing redaction rules for `sk-`, `api_key`, `token`, and `password`.

- [x] **Step 3: Redact unsafe URLs**

Remove credentials and secret query values from diagnostics.

### Task 4: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-model --test openai_reqwest_transport
cargo test -p agent-model --test openai_compatible
```

Expected: all focused tests pass without external network.

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

Expected: workspace passes.

- [ ] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates/agent-model docs
git commit -m "feat: add reqwest openai compatible transport"
```
