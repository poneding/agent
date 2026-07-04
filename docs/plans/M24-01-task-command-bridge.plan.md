# M24-01 Task Command Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add background task command contracts and thin CLI/TUI/Desktop-facing task summaries before runner execution.

**Architecture:** `agent-scheduler` owns task commands and transitions. `agent-ui-bridge` owns task summary schemas. CLI/TUI/Desktop render bridge data only.

**Tech Stack:** Rust 2024 edition, serde, existing `agent-scheduler` queue/state contracts, CLI JSON/text output, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M24-01-task-command-bridge.spec.md`
- Read: `docs/specs/M24-00-background-agents-technical-design.spec.md`
- Read: `crates/agent-scheduler/src/task.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-cli/src/command.rs`

- [x] **Step 1: Review task command scope**

Run:

```bash
sed -n '1,260p' docs/specs/M24-01-task-command-bridge.spec.md
```

Expected: command contracts, UI summaries, CLI surface, and acceptance criteria are clear.

### Task 2: Write RED Tests

**Files:**
- Create: `crates/agent-scheduler/tests/task_command_bridge.rs`
- Create: `crates/agent-ui-bridge/tests/task_command_bridge.rs`
- Create: `crates/agent-cli/tests/task_commands.rs`

- [x] **Step 1: Add scheduler command tests**

Cover create, list, inspect, pause, resume, cancel, update schedule, validation, and deterministic ordering.

- [x] **Step 2: Add UI bridge snapshot tests**

Cover task summary shape, redacted prompt summary, schedule labels, and command error shape.

- [x] **Step 3: Add CLI command tests**

Cover `agent task create|list|inspect|pause|resume|cancel|schedule` in text and JSON modes.

- [x] **Step 4: Verify RED**

Run:

```bash
cargo test -p agent-scheduler task_command_bridge
cargo test -p agent-ui-bridge task_command_bridge
cargo test -p agent-cli task_commands
```

Expected: tests fail because command bridge contracts are missing.

### Task 3: Implement Task Command Bridge

**Files:**
- Create: `crates/agent-scheduler/src/command.rs`
- Modify: `crates/agent-scheduler/src/lib.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-tui/src/task_panel.rs`
- Modify: `crates/agent-desktop/src/task.rs`

- [x] **Step 1: Add task command contracts**

Implement `BackgroundAgentTask`, task schedule, create request, command enum, command response, and summaries.

- [x] **Step 2: Add in-memory command service**

Implement create/list/inspect/pause/resume/cancel/update schedule using scheduler state transitions.

- [x] **Step 3: Add UI bridge summaries**

Normalize scheduler task records into UI-safe snapshots with bounded/redacted prompt summaries.

- [x] **Step 4: Add CLI/TUI/Desktop renderers**

Add task commands and task summary panels/models without putting mutation logic in UI crates.

### Task 4: Verify

**Files:**
- Read: `crates/agent-scheduler/tests/task_command_bridge.rs`
- Read: `crates/agent-cli/tests/task_commands.rs`

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-scheduler task_command_bridge
cargo test -p agent-ui-bridge task_command_bridge
cargo test -p agent-cli task_commands
```

Expected: focused tests pass.

- [x] **Step 2: Run workspace checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

### Task 5: Update Status and Commit

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M24-01-task-command-bridge.plan.md`
- Modify: `docs/plans/M24-background-agents-ux.plan.md`

- [x] **Step 1: Mark M24-01 complete**

Advance active implementation focus to M24-02.

- [x] **Step 2: Commit M24-01**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: add task command bridge"
```

Expected: commit succeeds.
