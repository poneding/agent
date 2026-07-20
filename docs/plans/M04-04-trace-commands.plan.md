# M04-04 Trace Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build deterministic trace query contracts and a CLI trace summary command.

**Architecture:** `ruse-core` converts runtime events into trace entries. `ruse-cli` renders trace entries as text and does not inspect storage internals.

**Tech Stack:** Rust 2024 edition, `ruse-core`, `ruse-cli`, `ruse-storage`, `ruse-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Core Trace Tests

**Files:**
- Create: `crates/ruse-core/tests/trace_query.rs`

- [x] **Step 1: Add failing trace tests**

Tests must prove tool/policy filtering, checkpoint rendering, final response rendering, empty trace output, and secret-like value redaction.

- [x] **Step 2: Verify RED**

```bash
cargo test -p ruse-core trace_query
```

Expected: tests fail because trace contracts do not exist.

### Task 2: Implement Trace Contracts

**Files:**
- Create: `crates/ruse-core/src/trace.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Define `TraceQuery` and `TraceEntry`**

Use the fields from `docs/specs/M04-04-trace-commands.spec.md`.

- [x] **Step 2: Implement trace builder**

Map runtime events into stable summaries and apply query filters.

- [x] **Step 3: Verify core GREEN**

```bash
cargo test -p ruse-core trace_query
```

Expected: focused trace tests pass.

### Task 3: Add CLI Trace Smoke

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/tests/trace_commands.rs`

- [x] **Step 1: Add failing CLI test**

Test `ruse-cli --trace-summary` prints deterministic trace entries for the local coding fixture.

- [x] **Step 2: Implement CLI renderer**

Render `TraceEntry` rows with `seq`, `kind`, and `summary`.

- [x] **Step 3: Verify CLI GREEN**

```bash
cargo test -p ruse-cli trace_commands
```

Expected: CLI trace test passes.

### Task 4: Run Verification and Commit

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M04-04-trace-commands.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [x] **Step 2: Advance active focus**

Update active focus from `M04-04-trace-commands` to `M04-05-replay-compatibility`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m04-04 trace commands"
```
