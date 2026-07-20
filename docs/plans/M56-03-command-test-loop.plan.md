# M56-03 Command/Test Loop Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed, policy-routed command/test loop that runs workspace verification commands, returns bounded/redacted output, classifies failures, exposes cancellation readiness, and feeds safe observations back into the model loop.

**Architecture:** `ruse-core` owns the command/test contract and orchestration. `ruse-tool` remains the process execution boundary through `ShellCommandTool`. `ruse-cli` renders command surfaces. `ruse-ui-bridge` owns safe command/test snapshots. `ruse-tui` consumes bridge evidence. Runtime logic must not move into UI crates.

**Tech Stack:** Rust 2024 edition, serde, existing `ToolLoop`, `WorkspaceSandbox`, `StaticPolicyEngine`, `ShellCommandTool`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-03-command-test-loop.spec.md`
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `crates/ruse-tool/src/builtins/shell_command.rs`
- Read: `crates/ruse-core/src/tool_loop.rs`
- Read: `crates/ruse-core/tests/comprehensive_turn_observations.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review command/test boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M56-03-command-test-loop.spec.md
```

Expected: scope is limited to command/test evidence, bounded output, cancellation readiness, classification, and observation feedback. Final review aggregation and full golden-path E2E remain later slices.

### Task 2: Write Failing Core Tests

**Files:**
- Create: `crates/ruse-core/tests/command_test_loop.rs`

- [x] **Step 1: Add core tests**

Cover:

- successful command returns status `Succeeded`, classification `None`, bounded stdout, and ordered tool-loop events
- non-zero command returns status `Failed` and classification `CommandFailure`
- `classify_as_test=true` maps non-zero status to `TestFailure`
- timeout returns status `TimedOut`, classification `Timeout`, and cancellation state that says the process has been stopped
- long stdout/stderr are bounded, marked truncated, and still preserve byte counts
- secret-like stdout/stderr/diagnostics are redacted
- invalid workspace or cwd rejects before command execution
- returned observation is bounded and suitable for model feedback

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-core command_test_loop
```

Expected: tests fail before the core command/test loop exists.

### Task 3: Implement Core Command/Test Loop

**Files:**
- Create: `crates/ruse-core/src/command_test.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add typed contracts**

Implement `CommandTestLoopInput`, `CommandTestLoopReport`, command intent, execution status, failure classification, bounded output, cancellation state, observation summary, diagnostics, and errors.

- [x] **Step 2: Orchestrate existing shell tool**

Use `ShellCommandTool` through `ToolLoop` with `workspace.shell_command`. Do not spawn commands directly from `ruse-core`.

- [x] **Step 3: Extract safe evidence**

Deserialize `ShellCommandResult`, classify status, bound/redact stdout/stderr, build cancellation readiness, and preserve ordered runtime events.

- [x] **Step 4: Run core checks**

Run:

```bash
cargo test -p ruse-tool shell_command_tool
cargo test -p ruse-core command_test_loop
cargo test -p ruse-core comprehensive_turn_observations
```

Expected: shell tool, command/test loop, and model observation checks pass.

### Task 4: Add UI Bridge and TUI Evidence

**Files:**
- Create: `crates/ruse-ui-bridge/tests/command_test_snapshot.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Create if needed: `crates/ruse-tui/src/command_panel.rs`
- Modify if needed: `crates/ruse-tui/src/lib.rs`
- Modify if needed: `crates/ruse-tui/src/render.rs`
- Create: `crates/ruse-tui/tests/command_panel.rs`

- [x] **Step 1: Add bridge snapshot tests**

Cover safe serialization, truncation, redaction, classification labels, command display bounds, and frame encoding.

- [x] **Step 2: Implement bridge snapshot**

Expose a UI-safe command/test snapshot derived from `CommandTestLoopReport`.

- [x] **Step 3: Ensure TUI can consume evidence**

Add or extend TUI state for command/test evidence while preserving existing shell and diff rendering behavior.

- [x] **Step 4: Run bridge/TUI checks**

Run:

```bash
cargo test -p ruse-ui-bridge command_test_snapshot
cargo test -p ruse-tui command_panel
```

Expected: UI evidence tests pass.

### Task 5: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/ruse-cli/tests/command_test_loop.rs`
- Modify: `crates/ruse-cli/src/command.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo command --workspace . -- <program> <args...>`
- `repo command --workspace . --cwd subdir --timeout-ms 30000 -- <program> <args...>`
- `repo test --workspace . -- <program> <args...>` classifies non-zero exits as test failures
- `--json repo test ...` emits command/test snapshot, normalized events, and done frame
- invalid workspace/cwd/argument shape exits with code `2`
- secret-like output and diagnostics are redacted/bounded

- [x] **Step 2: Implement CLI command**

Add parser and renderer using core command/test loop. Do not accept environment variable overrides in this slice.

- [x] **Step 3: Run CLI checks**

Run:

```bash
cargo test -p ruse-cli command_test_loop
```

Expected: CLI tests pass.

### Task 6: Verify and Commit

**Files:**
- Modify: `docs/plans/M56-03-command-test-loop.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-core -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-tool shell_command_tool
cargo test -p ruse-core command_test_loop
cargo test -p ruse-core comprehensive_turn_observations
cargo test -p ruse-cli command_test_loop
cargo test -p ruse-ui-bridge command_test_snapshot
cargo test -p ruse-tui command_panel
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for M56-03 source, tests, and docs, then:

```bash
git commit -m "feat: add m56 command test loop"
```

Expected: commit contains only M56-03 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Core command/test flow exists and uses tool-loop/tool-policy boundaries.
- [x] Success, command failure, test failure, timeout, policy, and tool failure outcomes are classified.
- [x] Command stdout/stderr and observations are bounded and redacted.
- [x] Cancellation readiness is explicit in the report.
- [x] CLI text and JSON surfaces exist.
- [x] UI bridge/TUI command evidence is safe and serializable.
- [x] Observation feedback remains compatible with the comprehensive model loop.
- [x] Focused fmt/clippy/tests pass.
