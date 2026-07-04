# M06-03 CLI JSON Mode Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--json` mode to the CLI and emit newline-delimited UI bridge frames for the local runtime fixture.

**Architecture:** `agent-cli` parses output mode and renders JSON through `agent-ui-bridge`; runtime fixture generation remains in `agent-core`. Bridge frame encoding lives in `agent-ui-bridge` so Desktop can reuse it later.

**Tech Stack:** Rust 2024 edition, `agent-cli`, `agent-ui-bridge`, serde_json, assert_cmd, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write JSON Frame RED Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/json_frame.rs`

- [x] **Step 1: Add failing frame tests**

Create tests named:

- `json_frame_serializes_event_with_type_tag`
- `json_frame_serializes_done_count`
- `json_frame_encode_returns_single_line`

The tests should import `UiJsonFrame` and `encode_json_frame`.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-ui-bridge json_frame
```

Expected: fails because frame contracts do not exist.

### Task 2: Implement Bridge JSON Frames

**Files:**
- Create: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`
- Modify: `crates/agent-ui-bridge/src/error.rs`

- [x] **Step 1: Add frame module**

Define:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum UiJsonFrame {
    Event(UiEventEnvelope),
    Done { event_count: u64 },
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },
}

pub fn encode_json_frame(frame: &UiJsonFrame) -> Result<String, UiBridgeError> {
    let mut line = serde_json::to_string(frame)?;
    line.push('\n');
    Ok(line)
}
```

Add `From<serde_json::Error>` for `UiBridgeError`.

- [x] **Step 2: Export frame contracts**

Update `lib.rs` to export:

```rust
pub mod frame;

pub use frame::{encode_json_frame, UiJsonFrame};
```

- [x] **Step 3: Verify frame GREEN**

Run:

```bash
cargo test -p agent-ui-bridge --test json_frame
```

Expected: all frame tests pass.

### Task 3: Write CLI JSON RED Tests

**Files:**
- Create: `crates/agent-cli/tests/json_mode.rs`

- [x] **Step 1: Add failing CLI tests**

Create tests named:

- `json_mode_parse_json_local_summary`
- `json_mode_renders_parseable_bridge_frames`
- `json_mode_binary_prints_ndjson_frames`

Expected behavior:

- `parse_cli_invocation(["--json", "--local-summary"])` returns JSON mode plus `CliCommand::LocalSummary`.
- `render_cli_invocation` returns JSON lines that deserialize into `UiJsonFrame`.
- The binary command `agent-cli --json --local-summary` prints at least one `event` frame and one `done` frame.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-cli json_mode
```

Expected: fails because `--json` mode contracts do not exist.

### Task 4: Implement CLI JSON Mode

**Files:**
- Modify: `crates/agent-cli/Cargo.toml`
- Modify: `crates/agent-cli/src/command.rs`
- Modify: `crates/agent-cli/src/main.rs`
- Modify: `crates/agent-cli/src/lib.rs`

- [x] **Step 1: Add dependency**

Add to `agent-cli` dependencies:

```toml
agent-ui-bridge = { path = "../agent-ui-bridge" }
```

- [x] **Step 2: Add invocation contracts**

Add:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliInvocation {
    pub command: CliCommand,
    pub output_mode: OutputMode,
}
```

Add `parse_cli_invocation`; keep `parse_cli_command` backward compatible by returning the command from text-mode parsing.

- [x] **Step 3: Add JSON renderer**

Add `render_cli_invocation`. For `OutputMode::Text`, delegate to `render_cli_command`. For `OutputMode::Json` and `CliCommand::LocalSummary`, run `agent_core::run_local_coding_fixture()`, normalize each runtime event with `normalize_runtime_event`, encode each `UiJsonFrame::Event`, and append `UiJsonFrame::Done { event_count }`.

- [x] **Step 4: Update main**

`main` should call `parse_cli_invocation` and `render_cli_invocation`.

- [x] **Step 5: Verify CLI GREEN**

Run:

```bash
cargo test -p agent-cli --test json_mode
```

Expected: all JSON mode tests pass and existing CLI tests still pass.

### Task 5: Verification and Progress

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `README.md`
- Modify: `docs/plans/M06-03-cli-json-mode.plan.md`

- [x] **Step 1: Run standard checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 2: Advance active focus**

Update active focus to `M06-04-bridge-snapshots`.

- [x] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m06 cli json mode"
```

Expected: commit succeeds with source, tests, and documentation together.
