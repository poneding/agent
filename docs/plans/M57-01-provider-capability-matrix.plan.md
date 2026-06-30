# M57-01 Provider Capability Matrix Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend provider capabilities into a typed product matrix with deterministic mismatch diagnostics and CLI/JSON config visibility.

**Architecture:** Capability matching lives in `agent-model`; CLI and UI bridge surfaces render summaries derived from the typed model contracts.

**Tech Stack:** Rust 2024 edition, serde contracts, existing provider registry/config tests, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M57-01-provider-capability-matrix.spec.md`
- Read: `crates/agent-model/src/registry.rs`
- Read: `crates/agent-model/src/config.rs`
- Read: `crates/agent-model/tests/provider_registry.rs`
- Read: `crates/agent-cli/tests/config_diagnostics.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Review current capability behavior**

Run:

```bash
cargo test -p agent-model provider_registry
cargo test -p agent-cli config_diagnostics
```

Expected: current streaming/tool/structured/context capability checks and config diagnostics pass.

### Task 2: Write Failing Tests

**Files:**
- Create: `crates/agent-model/tests/provider_capability_matrix.rs`
- Modify: `crates/agent-model/tests/provider_registry.rs`
- Modify: `crates/agent-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Add agent-model capability tests**

Cover:

- extended `ModelCapabilities` serialization
- missing image/output/context capability diagnostics
- deterministic missing capability order
- default profiles for fake and OpenAI-compatible providers

- [x] **Step 2: Add CLI config diagnostics tests**

Cover:

- text config check includes provider capability summary
- JSON config snapshot includes parseable provider capability metadata
- secret-like provider values remain redacted

- [x] **Step 3: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-cli config_diagnostics
```

Expected: tests fail before capability matrix implementation exists.

### Task 3: Implement Capability Matrix

**Files:**
- Modify: `crates/agent-model/src/registry.rs`
- Modify if needed: `crates/agent-model/src/config.rs`
- Modify: `crates/agent-model/src/lib.rs`

- [x] **Step 1: Extend typed capability contracts**

Add image input, output token, and optional cost metadata while preserving serde compatibility.

- [x] **Step 2: Add matching and diagnostics helpers**

Expose deterministic missing capability labels and keep mismatch diagnostics typed.

- [x] **Step 3: Add default capability profiles**

Provide stable profiles for fake and OpenAI-compatible providers.

### Task 4: Surface Capabilities in CLI/Bridge

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-ui-bridge/src/frame.rs`

- [x] **Step 1: Render text summary**

Include concise capability summaries in `--config-check` output.

- [x] **Step 2: Render JSON summary**

Expose parseable provider capability metadata through existing config snapshot frames.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M57-01-provider-capability-matrix.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-model -p agent-cli -p agent-ui-bridge --all-targets -- -D warnings
cargo test -p agent-model provider_capability_matrix
cargo test -p agent-model provider_registry
cargo test -p agent-cli config_diagnostics
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging, then:

```bash
git commit -m "feat: add m57 provider capability matrix"
```

Expected: commit contains capability contracts, CLI/JSON summaries, tests, and docs/status updates.

### Completion Checklist

- [x] Capability matrix fields are typed and serde-tested.
- [x] Requirement matching includes image input and output token needs.
- [x] Missing capability diagnostics are deterministic and redacted.
- [x] Config check text/JSON exposes provider capability summaries.
- [x] Focused fmt/clippy/tests pass.
