# M15-04 CLI Prompt Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make text-mode `agent-cli --prompt` execute a config-driven turn.

**Architecture:** CLI continues to parse flags and render output, but delegates config loading and turn execution to `agent-core`. CLI builds a deterministic fake-provider registry for local execution in M15.

**Tech Stack:** Rust 2024 edition, existing CLI parser style, fake model provider fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M15-04-cli-prompt-runtime.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/tests/cli_modes.rs`
- Read: `crates/agent-cli/tests/cli_smoke.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,220p' docs/specs/M15-04-cli-prompt-runtime.spec.md
```

Expected: the spec defines prompt execution behavior, config handling, and privacy requirements.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-cli/tests/prompt_runtime.rs`
- Modify: `crates/agent-cli/tests/cli_modes.rs`

- [x] **Step 1: Add focused tests**

Add tests named:

- `cli_prompt_runtime_runs_default_fake_turn`
- `cli_prompt_runtime_uses_configured_fake_provider`
- `cli_prompt_runtime_rejects_unregistered_openai_provider_safely`
- `cli_prompt_runtime_redacts_secret_like_values`

Update existing prompt render assertions so prompt output is a runtime final response instead of a static echo.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-cli --test prompt_runtime
```

Expected: tests fail because CLI prompt runtime still does not call configured turn execution.

### Task 3: Implement CLI Runtime Prompt

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/lib.rs` if new helpers are public

- [x] **Step 1: Build effective config for prompt**

Reuse config-check loading/layering helpers and explicit provider/model overrides.

- [x] **Step 2: Build fake provider registry**

Register executable fake providers from effective config. Use deterministic script text based on the prompt, such as `fake response: <prompt>`.

- [x] **Step 3: Run configured turn and render final text**

Call `ConfiguredTurnRunner` and render only final response in text mode.

- [x] **Step 4: Verify focused tests**

Run:

```bash
cargo test -p agent-cli --test prompt_runtime
```

Expected: focused tests pass.

### Task 4: Verify And Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M15-04-cli-prompt-runtime.plan.md`
- Modify: `docs/plans/M15-config-driven-turn-execution.plan.md`

- [x] **Step 1: Run gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m15 cli prompt runtime"
```

Expected: commit succeeds.
