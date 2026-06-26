# M13-05 CLI Model Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI provider/model selection flags and UI-safe output.

**Architecture:** Keep parsing and presentation in `agent-cli`; use `agent-model` config contracts for validation. Do not add live provider invocation in this slice.

**Tech Stack:** Rust 2024 edition, existing CLI parser style, serde_json for JSON mode, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M13-05-cli-model-selection.spec.md`
- Read: `crates/agent-cli/src/lib.rs`
- Read: `crates/agent-cli/tests/cli_modes.rs`
- Read: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M13-05-cli-model-selection.spec.md
```

Expected: the spec defines provider/model CLI selection and JSON output behavior.

### Task 2: Write RED CLI Tests

**Files:**
- Modify: `crates/agent-cli/tests/cli_modes.rs`
- Modify: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Add focused tests**

Add tests:

- `cli_modes_parse_provider_and_model_flags`
- `cli_modes_reject_empty_provider_and_model_flags`
- `json_mode_renders_selected_provider_and_model`
- `json_mode_does_not_render_secret_references`

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-cli --test cli_modes --test json_mode
```

Expected: tests fail because CLI selection is not implemented.

### Task 3: Implement CLI Selection

**Files:**
- Modify: `crates/agent-cli/src/lib.rs`
- Modify: `crates/agent-cli/src/main.rs`

- [x] **Step 1: Extend CLI options**

Add provider/model flags and validation that rejects empty values.

- [x] **Step 2: Render UI-safe selection**

Include selected provider/model in JSON mode and avoid secret reference output.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M13-05-cli-model-selection.plan.md`
- Modify: `docs/plans/M13-real-model-providers-config.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo test -p agent-cli --test cli_modes --test json_mode
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m13 cli model selection"
```

Expected: commit succeeds.
