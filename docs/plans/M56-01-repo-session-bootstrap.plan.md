# M56-01 Repo Session Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a first-class repository session bootstrap path that future M56 coding-agent slices can build on.

**Architecture:** `ruse-core` owns repo session bootstrap contracts, diagnostics, and runtime events. `ruse-cli` owns command parsing and rendering. `ruse-ui-bridge` owns UI-safe frames/snapshots. Keep side-effecting edit/command behavior out of this slice.

**Tech Stack:** Rust 2024 edition, serde, existing config/provider primitives, std `Command` for read-only Git status if no existing helper fits, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-01-repo-session-bootstrap.spec.md`
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `crates/ruse-types/src/event.rs`
- Read: `crates/ruse-core/src/config.rs`
- Read: `crates/ruse-core/src/runtime.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/event.rs`

- [x] **Step 1: Review contracts and existing boundaries**

Run:

```bash
sed -n '1,260p' docs/specs/M56-01-repo-session-bootstrap.spec.md
```

Expected: bootstrap scope is limited to workspace/config/Git/provider/session evidence.

### Task 2: Write Failing Core Tests

**Files:**
- Create: `crates/ruse-core/tests/repo_session_bootstrap.rs`

- [x] **Step 1: Add core bootstrap tests**

Cover:

- valid Git workspace produces workspace/session/conversation events
- dirty Git workspace reports dirty state without mutating files
- non-Git workspace reports warning but succeeds
- invalid workspace root fails without events
- config/provider diagnostics are redacted

- [x] **Step 2: Run focused tests and verify failure**

Run:

```bash
cargo test -p ruse-core repo_session_bootstrap
```

Expected: tests fail before implementation exists.

### Task 3: Implement Core Bootstrap Contract

**Files:**
- Create: `crates/ruse-core/src/repo_session.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add typed contracts**

Implement input, report, Git summary, provider summary, config summary, diagnostics, and error types.

- [x] **Step 2: Add bootstrap function**

Implement read-only workspace validation, Git status detection, config/provider summary, runtime ids, and workspace/session/conversation events.

- [x] **Step 3: Run core tests**

Run:

```bash
cargo test -p ruse-core repo_session_bootstrap
```

Expected: core tests pass.

### Task 4: Add CLI Text and JSON Surface

**Files:**
- Create: `crates/ruse-cli/tests/repo_session_bootstrap.rs`
- Modify: `crates/ruse-cli/src/command.rs`
- Modify if needed: `crates/ruse-cli/src/output.rs`

- [x] **Step 1: Add CLI tests**

Cover:

- `repo session --workspace .` text summary
- `--json repo session --workspace .` parseable redacted frames
- invalid workspace exits with code `2`
- provider/config diagnostics do not expose secrets

- [x] **Step 2: Implement CLI command**

Add parser and renderer that call `ruse-core` bootstrap and use existing `CliOutput` patterns.

- [x] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p ruse-cli repo_session_bootstrap
```

Expected: CLI tests pass.

### Task 5: Add UI Bridge Evidence

**Files:**
- Create: `crates/ruse-ui-bridge/tests/repo_session_snapshot.rs`
- Modify if needed: `crates/ruse-ui-bridge/src/event.rs`
- Modify if needed: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Add bridge tests**

Cover UI-safe session/bootstrap serialization and redaction.

- [x] **Step 2: Implement bridge mapping only if existing events are insufficient**

Prefer existing `Workspace`, `Session`, `Progress`, `ConfigSnapshot`, or safe frame types before adding new wire shapes.

- [x] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p ruse-ui-bridge repo_session_snapshot
```

Expected: bridge tests pass.

### Task 6: Verify and Commit

**Files:**
- Modify: `docs/plans/M56-01-repo-session-bootstrap.plan.md`
- Modify: `docs/milestones/README.md`

- [x] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p ruse-core -p ruse-cli -p ruse-ui-bridge --all-targets -- -D warnings
cargo test -p ruse-core repo_session_bootstrap
cargo test -p ruse-cli repo_session_bootstrap
cargo test -p ruse-ui-bridge repo_session_snapshot
```

Expected: focused checks pass.

- [x] **Step 2: Commit**

Run exact-path staging for the M56-01 source, tests, and docs, then:

```bash
git commit -m "feat: add m56 repo session bootstrap"
```

Expected: commit contains only M56-01 source, tests, and documentation/status updates.

### Completion Checklist

- [x] Core bootstrap contracts and diagnostics exist.
- [x] CLI text and JSON command exists.
- [x] UI bridge evidence is safe and serializable.
- [x] Git clean/dirty/non-Git/invalid root cases are tested.
- [x] Config/provider diagnostics are redacted.
- [x] Focused fmt/clippy/tests pass.
