# M59-03 Approval Queue and Permission Review Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed approval queue snapshots plus CLI/JSON and TUI approval review workflows for daily-driver terminal use.

**Architecture:** `ruse-ui-bridge` owns approval queue snapshot contracts and safe derivation from UI approval requests/events. `ruse-cli` renders queue inspection and keeps existing explicit approval command parsing. `ruse-tui` renders panel state and emits existing `UiCommand::ResolveApproval` effects. Policy and lease semantics remain in `ruse-policy`/runtime crates.

**Tech Stack:** Rust 2024 edition, serde contracts, existing approval events and commands, CLI text/JSON tests, TUI reducer/render tests, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M59-03-approval-queue-permission-review.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/security-policy.md`
- Read: `crates/ruse-ui-bridge/src/event.rs`
- Read: `crates/ruse-ui-bridge/src/command.rs`
- Read: `crates/ruse-cli/src/approval.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-tui/src/approval_panel.rs`

- [x] **Step 1: Review approval baseline**

Run:

```bash
cargo test -p ruse-ui-bridge --test approval_bridge
cargo test -p ruse-cli --test approval_prompt
cargo test -p ruse-tui --test approval_panel
```

Expected: existing approval bridge, CLI prompt, and TUI panel behavior pass before queue changes.

### Task 2: Add UI Bridge Approval Queue Tests

**Files:**
- Create: `crates/ruse-ui-bridge/tests/approval_queue_snapshot.rs`

- [x] **Step 1: Add bridge tests**

Cover:

- empty queue snapshot is successful
- pending approvals sort deterministically
- duplicate request ids collapse to latest request
- groups count by resource kind/action/status
- resolved/expired/cancelled/consumed events remove pending items
- text fields are bounded and redacted
- approval queue JSON frame encodes on one line

- [x] **Step 2: Verify bridge tests fail before implementation**

Run:

```bash
cargo test -p ruse-ui-bridge --test approval_queue_snapshot
```

Expected: tests fail before approval queue contracts exist.

### Task 3: Implement UI Bridge Approval Queue Contracts

**Files:**
- Create: `crates/ruse-ui-bridge/src/approval_queue.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`

- [x] **Step 1: Add queue snapshot contracts**

Implement queue item, group, diagnostic, snapshot, and queue reducer helpers using `UiApprovalRequest` and approval-resolved events.

- [x] **Step 2: Add deterministic derivation**

Implement sorting, duplicate collapse, group counts, stale/resolved removal, count summaries, and redaction/bounding.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge --test approval_queue_snapshot
```

Expected: bridge approval queue tests pass.

### Task 4: Add CLI Approval Queue Commands

**Files:**
- Modify: `crates/ruse-cli/src/approval.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Create: `crates/ruse-cli/tests/approval_queue_commands.rs`
- Modify if needed: `crates/ruse-cli/tests/approval_prompt.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `approval queue --fixture empty` text output exits `0`
- `approval queue --fixture sample` text output includes count, group, lease, preview, and redaction indicators
- `--json approval queue --fixture sample` emits `ApprovalQueueSnapshot` and `Done`
- expired/resolved fixture has no actionable pending items and stable diagnostics
- output is bounded and redacted
- existing approval prompt behavior remains unchanged

- [x] **Step 2: Implement CLI parsing and rendering**

Add `approval queue` command routing, fixture-backed queue construction, text rendering, JSON frame rendering, and safe error handling.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli --test approval_queue_commands
cargo test -p ruse-cli --test approval_prompt
```

Expected: approval queue CLI tests and existing prompt tests pass.

### Task 5: Improve TUI Approval Panel

**Files:**
- Modify: `crates/ruse-tui/src/approval_panel.rs`
- Modify: `crates/ruse-tui/src/app.rs`
- Modify: `crates/ruse-tui/src/render.rs`
- Modify: `crates/ruse-tui/tests/approval_panel.rs`

- [x] **Step 1: Add TUI tests**

Cover snapshot reduction, grouped metadata, queue count rendering, focus next/previous, allow/deny effects, empty state, resolved/expired removal, invalid id diagnostics, and lease display.

- [x] **Step 2: Implement panel updates**

Reduce `UiJsonFrame::ApprovalQueueSnapshot`, preserve event reduction compatibility, render queue count/groups, and keep all side effects as existing `TuiEffect` values.

- [x] **Step 3: Run TUI tests**

Run:

```bash
cargo test -p ruse-tui --test approval_panel
```

Expected: approval panel tests pass.

### Task 6: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M59-03-approval-queue-permission-review.plan.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-cli -p ruse-ui-bridge -p ruse-tui --all-targets -- -D warnings
cargo test -p ruse-ui-bridge --test approval_queue_snapshot
cargo test -p ruse-cli --test approval_queue_commands
cargo test -p ruse-cli --test approval_prompt
cargo test -p ruse-tui --test approval_panel
```

Expected: focused checks pass.

- [x] **Step 2: Run standard checks**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: workspace checks pass.

- [x] **Step 3: Commit**

Run exact-path staging for M59-03 source, tests, and docs, then:

```bash
git commit -m "feat: add m59 approval queue review"
```

Expected: commit contains only M59-03 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Approval queue snapshot contracts are typed, serde-friendly, deterministic, and tested.
- [x] Queue sorting, grouping, duplicate collapse, and resolved/expired removal are deterministic.
- [x] CLI text/JSON approval queue inspection is stable and redacted.
- [x] TUI approval panel displays queue count, groups, preview, lease, and redaction state.
- [x] Allow/deny/abort effects remain typed and validated.
- [x] Existing approval bridge and prompt behavior remains unchanged.
- [x] Focused and standard checks pass.
