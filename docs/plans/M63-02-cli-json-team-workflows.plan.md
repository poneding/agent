# M63-02 CLI and JSON Team Workflow Commands Plan

> **For agentic workers:** Follow this checklist task by task. Keep this slice focused on CLI/JSON and bridge-safe contracts. Do not implement live merge or subagent execution here.

**Goal:** Add stable CLI/JSON commands for inspecting and dry-running team workflows using the typed M63-01 `agent-team` snapshot contracts.

**Architecture:** `agent-team` owns workflow readiness and summaries. `agent-ui-bridge` owns JSON frame shape. `agent-cli` parses commands and renders typed snapshots. CLI does not classify readiness or reduce policy independently.

---

### Task 1: Plan Surfaces

**Files:**
- Create: `docs/specs/M63-02-cli-json-team-workflows.spec.md`
- Create: `docs/plans/M63-02-cli-json-team-workflows.plan.md`

- [x] **Step 1: Define command set**

Commands:

- `team status`
- `team handoff --objective <text> --role <role> --dry-run`
- `team blackboard`
- `team review --vote <vote> --summary <text> --dry-run`
- `team merge-readiness --dry-run`

- [x] **Step 2: Define JSON frame**

Add `UiJsonFrame::TeamWorkflowSnapshot` wrapping operation, dry-run flag, typed snapshot, and bounded command errors.

### Task 2: Add Bridge Contract

**Files:**
- Modify: `crates/agent-ui-bridge/Cargo.toml`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Create: `crates/agent-ui-bridge/tests/team_workflow_snapshot.rs`

- [x] **Step 1: Add bridge dependency on `agent-team`**

The bridge may reference the typed workflow snapshot directly.

- [x] **Step 2: Add frame wrapper**

Add `UiTeamWorkflowSnapshot` and `UiTeamWorkflowCommandError`.

- [x] **Step 3: Add frame test**

Verify stable JSON tag, operation, dry-run flag, snapshot readiness, and round-trip decode.

### Task 3: Add CLI Commands

**Files:**
- Modify: `crates/agent-cli/Cargo.toml`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/lib.rs`
- Create: `crates/agent-cli/tests/team_workflow_commands.rs`

- [x] **Step 1: Add command enums and parser**

Parse the M63-02 command set and reject side-effect-shaped commands without `--dry-run`.

- [x] **Step 2: Add deterministic runtime fixture**

Provide sample, conflict, and policy-blocked snapshots without provider credentials or durable state.

- [x] **Step 3: Add text and JSON rendering**

Text and JSON must derive from the same `UiTeamWorkflowSnapshot`.

- [x] **Step 4: Add focused tests**

Cover parse, text, JSON, handoff dry-run, review block, conflict readiness, policy-blocked readiness, and dry-run guard.

### Task 4: Verify

**Files:**
- Read: changed files

- [x] **Step 1: Format**

```bash
cargo fmt --all -- --check
```

- [x] **Step 2: Focused tests**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-ui-bridge --test team_workflow_snapshot
CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-cli --test team_workflow_commands
CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-tui
```

- [x] **Step 3: Focused clippy**

```bash
CARGO_TARGET_DIR=target/m63-gate cargo clippy -p agent-ui-bridge -p agent-cli -p agent-tui --all-targets -- -D warnings
```

- [x] **Step 4: Diff check**

```bash
git diff --check
```

### Task 5: Status And Commit

**Files:**
- Modify: `docs/plans/M63-worktree-agent-team-workflows.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Update status docs after focused gates pass**

Mark M63-02 implementation complete only after focused gates pass. Keep M63 overall in progress.

Result: focused gates passed with `cargo fmt --all -- --check`, `git diff --check`, `CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-ui-bridge --test team_workflow_snapshot`, `CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-cli --test team_workflow_commands`, `CARGO_TARGET_DIR=target/m63-gate cargo test -p agent-tui`, and `CARGO_TARGET_DIR=target/m63-gate cargo clippy -p agent-ui-bridge -p agent-cli -p agent-tui --all-targets -- -D warnings`.

- [ ] **Step 2: Commit**

Exact-stage only M63-02 files. Exclude `tmp/`.

Suggested commit message:

```text
feat: add m63 team workflow cli commands
```
