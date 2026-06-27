# M20-01 Agent Binary Entrypoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `agent` the primary binary command and route interactive no-arg runs toward TUI without breaking batch or JSON modes.

**Architecture:** `agent-cli` owns argument parsing and stdio mode dispatch. TUI launch is represented by a small boundary that M20-02 will implement behind `agent-tui`.

**Tech Stack:** Rust 2024 edition, `agent-cli`, injected TTY profile tests, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M20-01-agent-binary-entrypoint.spec.md`
- Read: `docs/specs/M20-00-terminal-ux-tui-technical-design.spec.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/src/bin/agent.rs`
- Read: `crates/agent-cli/src/bin/agent-cli.rs`
- Read: `crates/agent-cli/Cargo.toml`

- [x] **Step 1: Review dispatch boundary**

Run:

```bash
sed -n '1,220p' docs/specs/M20-01-agent-binary-entrypoint.spec.md
sed -n '1,220p' crates/agent-cli/src/command.rs
```

Expected: CLI dispatch remains presentation-only and runtime behavior stays in existing crates.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-cli/tests/agent_entrypoint.rs`

- [x] **Step 1: Add parser and stdio dispatch tests**

Cover `tui`, `repl --plain`, interactive no-arg TUI dispatch, non-TTY no-arg help failure, and JSON prompt dispatch.

- [x] **Step 2: Add binary smoke test**

Assert `env!("CARGO_BIN_EXE_agent")` runs `--version`. Keep existing `agent-cli` smoke tests green.

- [x] **Step 3: Verify RED**

Run:

```bash
cargo test -p agent-cli --test agent_entrypoint
```

Expected: fails because `agent` binary and dispatch contracts do not exist yet.

### Task 3: Implement Entrypoint

**Files:**
- Modify: `crates/agent-cli/Cargo.toml`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/lib.rs`
- Create: `crates/agent-cli/src/bin/agent.rs`
- Create: `crates/agent-cli/src/bin/agent-cli.rs`

- [x] **Step 1: Add binary targets**

Add `agent` as the primary bin and keep `agent-cli` as a compatibility alias if existing tests depend on it.

- [x] **Step 2: Add dispatch contracts**

Add `InteractiveMode`, `StdioProfile`, and command variants needed by this spec.

- [x] **Step 3: Preserve script modes**

Ensure `--json`, `--prompt`, `--config-check`, and `--version` use current render paths.

### Task 4: Verify

**Files:**
- Read: `crates/agent-cli/tests/agent_entrypoint.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-cli --test agent_entrypoint
```

Expected: focused tests pass.

- [x] **Step 2: Run CLI tests**

Run:

```bash
cargo test -p agent-cli
```

Expected: existing CLI behavior remains green.

### Task 5: Update Status

**Files:**
- Modify: `docs/plans/M20-01-agent-binary-entrypoint.plan.md`
- Modify: `docs/plans/M20-terminal-ux-tui-repl.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Mark M20-01 complete**

Update status after verification and set current focus to M20-02.
