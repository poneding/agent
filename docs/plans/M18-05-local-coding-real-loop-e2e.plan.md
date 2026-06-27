# M18-05 Local Coding Real Loop E2E Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the comprehensive turn runner can complete a deterministic local coding workflow using real built-in tools.

**Architecture:** The test wires `ComprehensiveTurnRunner` to the real tool registry, a fixture allowlist policy, a temporary workspace, and a scripted fake model. Assertions use emitted runtime events rather than calling tool internals directly.

**Tech Stack:** Rust 2024 edition, Tokio tests, tempfile, existing local coding tools, fake model provider, event replay/trace helpers, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Write RED Local Coding E2E Test

**Files:**
- Create: `crates/agent-core/tests/comprehensive_local_coding_e2e.rs`

- [x] **Step 1: Add full workflow test**

Add `comprehensive_turn_completes_local_coding_workflow`.

```rust
#[tokio::test]
async fn comprehensive_turn_completes_local_coding_workflow() {
    let fixture = local_coding_fixture("hello\n");

    let outcome = ComprehensiveTurnRunner::run_with_idempotency(
        fixture.input,
        &fixture.models,
        fixture.resources,
        &fixture.idempotency_store,
    )
    .await
    .expect("local coding turn");

    assert_eq!(outcome.terminal_state, AgentLoopTerminalState::Done);
    assert_eq!(
        std::fs::read_to_string(fixture.workspace.path().join("notes.txt"))
            .expect("edited file"),
        "hello from agent\n"
    );
    assert_eq!(outcome.final_message.as_deref(), Some("Updated notes.txt and verified it."));
}
```

- [x] **Step 2: Add event trace assertions**

In the same test, assert that the runtime trace includes the expected tool phases.

```rust
let phases: Vec<&'static str> = outcome
    .events
    .iter()
    .filter_map(|event| match event.payload {
        RuntimeEventPayload::ToolCallProposed(_) => Some("proposed"),
        RuntimeEventPayload::PermissionDecisionRecorded(_) => Some("decision"),
        RuntimeEventPayload::ToolExecutionStarted(_) => Some("started"),
        RuntimeEventPayload::ToolExecutionCompleted(_) => Some("completed"),
        RuntimeEventPayload::ToolObservationRecorded(_) => Some("observed"),
        RuntimeEventPayload::FinalResponseProduced(_) => Some("final"),
        _ => None,
    })
    .collect();

assert!(phases.windows(5).any(|window| {
    window == ["proposed", "decision", "started", "completed", "observed"]
}));
assert_eq!(phases.last(), Some(&"final"));
```

- [x] **Step 3: Run proof test**

Run:

```bash
cargo test -p agent-core --test comprehensive_local_coding_e2e
```

Expected: PASS when M18-01 through M18-04 already provide the complete comprehensive loop path. If it fails, fix the comprehensive loop rather than bypassing it in the fixture.

### Task 2: Build Fixture Tool Registry

**Files:**
- Modify: `crates/agent-core/tests/comprehensive_local_coding_e2e.rs`

- [x] **Step 1: Create temporary workspace**

Use `tempfile::tempdir()` and write `notes.txt` containing `hello\n`.

- [x] **Step 2: Register real tools**

Create a test helper that registers the existing read, edit, shell, and Git inspection tools with workspace confinement set to the temporary directory.

- [x] **Step 3: Add allowlist policy fixture**

Use an explicit policy fixture that allows only the test tool names and the temporary workspace path.

### Task 3: Build Fake Model Script

**Files:**
- Modify: `crates/agent-core/tests/comprehensive_local_coding_e2e.rs`
- Modify: `crates/agent-model/src/fake.rs` only if the existing fake provider cannot express the required sequence

- [x] **Step 1: Script read request**

The first model turn proposes a read tool call for `notes.txt`.

```rust
FakeModelScript::tool_call(
    "read-1",
    "workspace.read_file",
    serde_json::json!({ "path": "notes.txt" }),
)
```

- [x] **Step 2: Script edit request**

The second model turn proposes an edit tool call that changes file content to `hello from agent\n`.

- [x] **Step 3: Script shell verification**

The third model turn proposes a shell command that verifies the file contains `hello from agent`.

- [x] **Step 4: Script Git inspection and final response**

The fourth model turn proposes Git status or diff inspection. The final model turn returns `Updated notes.txt and verified it.`

### Task 4: Add Replay or Trace Reconstruction Assertion

**Files:**
- Modify: `crates/agent-core/tests/comprehensive_local_coding_e2e.rs`
- Modify: `crates/agent-core/src/trace.rs` only if an existing public trace helper cannot express the assertion

- [x] **Step 1: Reconstruct final response from events**

Use emitted events to find the final response without reading runner internals.

```rust
let replayed_final = outcome.events.iter().find_map(|event| match &event.payload {
    RuntimeEventPayload::FinalResponseProduced(response) => Some(response.message.as_str()),
    _ => None,
});
assert_eq!(replayed_final, Some("Updated notes.txt and verified it."));
```

- [x] **Step 2: Assert tool names from events**

Collect proposed tool names from `ToolCallProposed` events and assert the sequence contains read, edit, shell, and Git inspection names.

### Task 5: Complete M18 Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M18-agent-tool-loop-runtime.plan.md`

- [x] **Step 1: Mark M18 slice implementation complete**

Update M18 status rows to mark M18-01 through M18-05 complete.

- [x] **Step 2: Advance active focus**

Set active implementation focus to `M19 Interactive Approval and Safe Local Actions`.

### Task 6: Verify

**Files:**
- Read: focused test output

- [x] **Step 1: Run focused tests**

Run:

```bash
cargo test -p agent-core --test comprehensive_local_coding_e2e
cargo test -p agent-core --test comprehensive_turn_guards
cargo test -p agent-core --test comprehensive_turn_idempotency
```

- [x] **Step 2: Run workspace gate**

Run:

```bash
cargo xtask check
```

- [ ] **Step 3: Commit**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m18 agent tool loop"
```
