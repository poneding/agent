# M57 Multi-Provider Runtime Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make model provider choice typed, diagnosable, and product-ready across OpenAI-compatible, Anthropic, Gemini, and local/Ollama-style providers while preserving the M56 golden path.

**Architecture:** Provider contracts and transports live in `agent-model`; runtime config composition remains in `agent-core`; CLI/JSON and bridge surfaces consume typed summaries without owning provider logic.

**Tech Stack:** Rust 2024 edition, serde contracts, async model streams, fixture transports, Cargo fmt/clippy/test, `cargo xtask eval`.

---

### Task 1: Confirm M57 Inputs

**Files:**
- Read: `docs/specs/M57-multi-provider-runtime.spec.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`
- Read: `crates/agent-model/src/config.rs`
- Read: `crates/agent-model/src/registry.rs`
- Read: `crates/agent-model/src/invocation_policy.rs`
- Read: `crates/agent-cli/tests/live_provider_registration.rs`

- [ ] **Step 1: Review provider baseline**

Run:

```bash
cargo test -p agent-model provider_registry
cargo test -p agent-cli live_provider_registration
```

Expected: existing fake/OpenAI-compatible provider registry behavior passes before M57 changes.

### Task 2: M57-01 Provider Capability Matrix

**Files:**
- Execute: `docs/plans/M57-01-provider-capability-matrix.plan.md`

- [x] **Step 1: Complete capability matrix slice**

Add typed provider/model capability profiles, mismatch diagnostics, default profiles for current providers, and CLI/JSON exposure.

### Task 3: Provider Adapters

**Files:**
- Execute: `docs/plans/M57-02-anthropic-adapter.plan.md`
- Execute: `docs/plans/M57-03-gemini-adapter.plan.md`
- Create later: `docs/specs/M57-04-local-provider-adapter.spec.md`
- Create later: `docs/plans/M57-04-local-provider-adapter.plan.md`

- [x] **Step 1: Add native Anthropic adapter**

Implement fixture-backed Claude streaming/tool-use mapping with safe diagnostics.

- [x] **Step 2: Add native Gemini adapter**

Implement fixture-backed Gemini streaming/tool-use mapping with safe diagnostics.

- [ ] **Step 3: Add local/Ollama-compatible provider**

Implement local endpoint profile, local HTTP safety checks, and diagnostics.

### Task 4: Provider UX and Fallback

**Files:**
- Create later: `docs/specs/M57-05-provider-ux-fallback.spec.md`
- Create later: `docs/plans/M57-05-provider-ux-fallback.plan.md`

- [ ] **Step 1: Complete provider UX/fallback slice**

Expose provider capability diagnostics through CLI/JSON, ensure fallback only selects compatible providers, and add redacted support output.

### Task 5: Final Verification and Closeout

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M57-multi-provider-runtime.plan.md`

- [ ] **Step 1: Run final M57 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all gates pass with provider diagnostics redacted.

- [ ] **Step 2: Commit M57 closeout**

Run exact-path staging and commit the final docs/status updates.

### Completion Checklist

- [x] Provider capability matrix is typed, serde-friendly, and tested.
- [x] Anthropic adapter is implemented with fixture-backed tests.
- [x] Gemini adapter is implemented with fixture-backed tests.
- [ ] Local/Ollama-compatible provider is implemented with local endpoint safety.
- [ ] CLI/JSON provider diagnostics and fallback behavior are tested.
- [ ] M56 golden path remains green under `cargo xtask eval`.
- [ ] Final workspace fmt/clippy/tests pass.
