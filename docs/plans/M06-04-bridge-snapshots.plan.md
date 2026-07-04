# M06-04 Bridge Snapshots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lock M06 UI bridge wire contracts with deterministic snapshot-style tests and close the M06 milestone.

**Architecture:** Snapshot tests live in `agent-ui-bridge` because it owns event, command, and frame wire shapes. CLI JSON mode reuses bridge frames and only tests that its output remains parseable.

**Tech Stack:** Rust 2024 edition, `agent-ui-bridge`, `agent-cli`, serde_json fixtures, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write Bridge Snapshot RED Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/bridge_snapshots.rs`
- Create: `crates/agent-ui-bridge/tests/fixtures/m06_model_delta_event.json`
- Create: `crates/agent-ui-bridge/tests/fixtures/m06_start_turn_command.json`
- Create: `crates/agent-ui-bridge/tests/fixtures/m06_done_frame.json`

- [x] **Step 1: Add fixture files**

Fixture files use schema version `1`, nil-like deterministic UUID strings, and no real secrets.

- [x] **Step 2: Add failing snapshot tests**

Create tests named:

- `bridge_snapshots_match_model_delta_event_fixture`
- `bridge_snapshots_match_start_turn_command_fixture`
- `bridge_snapshots_match_done_frame_fixture`
- `bridge_snapshots_redact_tool_call_input_from_ui_event`

- [x] **Step 3: Verify RED or fixture mismatch**

Run:

```bash
cargo test -p agent-ui-bridge bridge_snapshots
```

Expected: fails until fixture shape and bridge serialization agree.

### Task 2: Align Snapshot Serialization

**Files:**
- Modify: `crates/agent-ui-bridge/src/event.rs`
- Modify: `crates/agent-ui-bridge/src/command.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/tests/fixtures/*.json`

- [x] **Step 1: Normalize serialized field order through serde_json values**

Snapshot tests should compare `serde_json::Value`, not raw strings, so formatting does not cause false failures.

- [x] **Step 2: Keep compatibility tags stable**

Ensure public enums retain:

```rust
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
```

- [x] **Step 3: Verify bridge snapshots GREEN**

Run:

```bash
cargo test -p agent-ui-bridge --test bridge_snapshots
```

Expected: all bridge snapshot tests pass.

### Task 3: Add CLI JSON Contract Check

**Files:**
- Modify: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Add full-stream assertion**

Extend the CLI JSON test so every stdout line from `agent-cli --json --local-summary` deserializes into `agent_ui_bridge::UiJsonFrame`, the final line is `Done`, and at least one event line is present before it.

- [x] **Step 2: Verify CLI JSON contract**

Run:

```bash
cargo test -p agent-cli --test json_mode
```

Expected: CLI JSON stream remains parseable and ends with `Done`.

### Task 4: M06 Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M06-04-bridge-snapshots.plan.md`
- Modify: `README.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Mark M06 complete**

Update `docs/milestones/README.md` so M06 implementation is complete and active focus advances to `M07-mcp-skills`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: complete m06 json ui bridge"
```

Expected: commit succeeds with source, tests, and documentation together.
