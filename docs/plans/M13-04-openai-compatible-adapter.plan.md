# M13-04 OpenAI-Compatible Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fixture-backed OpenAI-compatible provider adapter for request mapping and streaming response parsing.

**Architecture:** Keep provider-specific JSON and SSE parsing inside `ruse-model::openai_compatible`. Use fixture transports in tests and do not require live network calls or real secrets.

**Tech Stack:** Rust 2024 edition, serde_json, async-trait, thiserror, existing `ModelProvider` trait, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M13-04-openai-compatible-adapter.spec.md`
- Read: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`
- Read: `crates/ruse-model/src/request.rs`
- Read: `crates/ruse-model/src/stream.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,240p' docs/specs/M13-04-openai-compatible-adapter.spec.md
```

Expected: the spec defines fixture-backed request mapping and stream parsing.

### Task 2: Write RED Adapter Tests

**Files:**
- Create: `crates/ruse-model/tests/openai_compatible.rs`

- [x] **Step 1: Add focused tests**

Add tests:

- `openai_compatible_request_builder_emits_responses_payload`
- `openai_compatible_provider_requires_secret_ref`
- `openai_compatible_stream_parser_maps_delta_and_completed_events`
- `openai_compatible_stream_parser_ignores_done_marker`
- `openai_compatible_errors_redact_response_bodies`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-model --test openai_compatible
```

Expected: tests fail because adapter contracts do not exist.

### Task 3: Implement Adapter

**Files:**
- Create: `crates/ruse-model/src/openai_compatible.rs`
- Modify: `crates/ruse-model/src/lib.rs`

- [x] **Step 1: Add request builder**

Build Responses API-style JSON containing `model`, `input`, and `stream`.

- [x] **Step 2: Add fixture transport and parser**

Implement a fixture transport trait and parse synthetic SSE chunks into `ModelStreamEvent` values.

- [x] **Step 3: Add provider wrapper**

Implement `OpenAiCompatibleProvider` behind `ModelProvider` using fixture transport for tests.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-04-openai-compatible-adapter.plan.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo test -p ruse-model --test openai_compatible
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 openai compatible adapter"
```

Expected: commit succeeds.
