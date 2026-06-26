# M14-04 CLI Config Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI config-check behavior and UI-safe JSON diagnostics.

**Architecture:** `agent-cli` parses config flags and delegates validation to `agent-core::config`. JSON output uses `agent-ui-bridge` frames; CLI does not parse TOML internals directly.

**Tech Stack:** Rust 2024 edition, existing CLI parser style, tempfile for tests if already available, serde_json, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M14-04-cli-config-diagnostics.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/cli_modes.rs`
- Read: `crates/agent-cli/tests/json_mode.rs`

- [ ] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M14-04-cli-config-diagnostics.spec.md
```

Expected: the spec defines config flags, diagnostics, JSON behavior, and privacy requirements.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-cli/tests/config_diagnostics.rs`

- [ ] **Step 1: Add focused tests**

Add tests named:

- `cli_config_diagnostics_parse_config_check`
- `cli_config_diagnostics_text_reports_valid_config`
- `cli_config_diagnostics_json_renders_config_summary`
- `cli_config_diagnostics_invalid_toml_exits_non_zero`
- `cli_config_diagnostics_redacts_secret_like_values`

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-cli --test config_diagnostics
```

Expected: tests fail because config-check CLI behavior does not exist.

### Task 3: Implement CLI Diagnostics

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/lib.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`

- [ ] **Step 1: Extend CLI invocation**

Add:

- optional `config_path`
- `CliCommand::ConfigCheck`
- `--config <path>`
- `--config-check`

- [ ] **Step 2: Render text and JSON diagnostics**

Text mode prints a concise config summary. JSON mode emits parseable frames and never includes raw secret values.

- [ ] **Step 3: Verify focused tests**

Run:

```bash
cargo test -p agent-cli --test config_diagnostics
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M14-04-cli-config-diagnostics.plan.md`
- Modify: `docs/plans/M14-runtime-configuration-secrets.plan.md`
- Modify: `README.md`

- [ ] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [ ] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m14 cli config diagnostics"
```

Expected: commit succeeds.
