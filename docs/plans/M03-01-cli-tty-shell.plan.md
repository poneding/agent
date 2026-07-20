# M03-01 CLI TTY Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a testable CLI command surface with version, banner, batch prompt, TTY shell, and slash command parsing.

**Architecture:** `ruse-cli` gets a library boundary for parsing and output formatting. `main.rs` becomes a thin adapter over library functions, while runtime logic stays in `ruse-core`.

**Tech Stack:** Rust 2024 edition, `ruse-core`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Add CLI Library Boundary

**Files:**
- Create: `crates/ruse-cli/src/lib.rs`
- Create: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/src/output.rs`
- Modify: `crates/ruse-cli/src/main.rs`

- [x] **Step 1: Write failing parser tests**

Create `crates/ruse-cli/tests/cli_modes.rs` with tests for:

- `--version`
- no args runtime banner
- `--prompt "hello"`
- `--tty`
- unknown flag error

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-cli cli_modes
```

Expected: tests fail because `ruse_cli::CliCommand` and parser functions do not exist.

- [x] **Step 3: Implement parser and output contracts**

Implement `CliCommand`, `CliOutput`, and `parse_cli_command`.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p ruse-cli cli_modes
```

Expected: parser tests pass.

### Task 2: Add Slash Commands

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/tests/slash_commands.rs`

- [x] **Step 1: Write failing slash tests**

Tests must prove `/help`, `/status`, `/exit`, and unknown slash commands return deterministic `CliOutput`.

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-cli slash_commands
```

Expected: tests fail before slash command parser exists.

- [x] **Step 3: Implement slash command parsing**

Implement `SlashCommand` and `handle_slash_command`.

- [x] **Step 4: Verify GREEN**

```bash
cargo test -p ruse-cli slash_commands
```

Expected: slash command tests pass.

### Task 3: Wire Binary Modes

**Files:**
- Modify: `crates/ruse-cli/src/main.rs`
- Modify: `crates/ruse-cli/tests/cli_smoke.rs`
- Modify: `crates/ruse-cli/tests/runtime_spine.rs`

- [x] **Step 1: Add binary smoke tests**

Tests must prove `ruse-cli --version`, `ruse-cli`, and `ruse-cli --prompt hello` print exact expected output.

- [x] **Step 2: Implement binary adapter**

`main.rs` should call `parse_cli_command` and print `CliOutput.stdout`.

- [x] **Step 3: Run focused CLI tests**

```bash
cargo test -p ruse-cli
```

Expected: all CLI tests pass.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M03-01-cli-tty-shell.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M03-01-cli-tty-shell` to `M03-02-file-edit-tool`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m03-01 cli tty shell"
```
