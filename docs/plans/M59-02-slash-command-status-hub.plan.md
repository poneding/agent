# M59-02 Slash Command Catalog and Status Hub Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a shared typed slash command catalog and bounded status hub for CLI/plain REPL and TUI daily-driver surfaces.

**Architecture:** `agent-ui-bridge` owns UI-safe catalog/status contracts. `agent-cli` renders plain REPL slash output from those contracts. `agent-tui` parses catalog-backed slash commands and routes them to local panel/status state. Runtime side effects stay out of the slash parser.

**Tech Stack:** Rust 2024 edition, serde contracts, existing `agent-types` ids where needed, CLI text tests, TUI reducer/render tests, Cargo fmt/clippy/test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M59-02-slash-command-status-hub.spec.md`
- Read: `docs/specs/M59-cli-tui-daily-driver.spec.md`
- Read: `docs/architecture/crate-boundaries.md`
- Read: `crates/agent-cli/src/command.rs`
- Read: `crates/agent-cli/src/repl.rs`
- Read: `crates/agent-ui-bridge/src/frame.rs`
- Read: `crates/agent-tui/src/slash.rs`
- Read: `crates/agent-tui/src/app.rs`

- [ ] **Step 1: Review slash/status baseline**

Run:

```bash
cargo test -p agent-cli --test slash_commands
cargo test -p agent-cli --test plain_repl
cargo test -p agent-tui --test slash_commands
```

Expected: existing slash and plain REPL behavior passes before catalog changes.

### Task 2: Add UI Bridge Catalog and Status Tests

**Files:**
- Create: `crates/agent-ui-bridge/tests/slash_catalog.rs`

- [ ] **Step 1: Add bridge tests**

Cover:

- deterministic catalog ordering
- aliases resolve to canonical command names
- descriptors serialize with category, summary, effect, and surfaces
- status snapshots bound and redact public text
- JSON frames encode catalog/status snapshots on one line

- [ ] **Step 2: Verify bridge tests fail before implementation**

Run:

```bash
cargo test -p agent-ui-bridge --test slash_catalog
```

Expected: tests fail before slash catalog contracts exist.

### Task 3: Implement UI Bridge Catalog and Status Contracts

**Files:**
- Create if needed: `crates/agent-ui-bridge/src/slash.rs`
- Modify: `crates/agent-ui-bridge/src/frame.rs`
- Modify: `crates/agent-ui-bridge/src/lib.rs`

- [ ] **Step 1: Add typed catalog contracts**

Implement descriptor, category, effect, surface, catalog snapshot, alias lookup, and deterministic default catalog helpers.

- [ ] **Step 2: Add terminal status snapshot**

Implement bounded/redacted terminal status summary and JSON frame variants.

- [ ] **Step 3: Run bridge tests**

Run:

```bash
cargo test -p agent-ui-bridge --test slash_catalog
```

Expected: bridge catalog/status tests pass.

### Task 4: Add CLI and Plain REPL Catalog Rendering

**Files:**
- Modify: `crates/agent-cli/src/command.rs`
- Modify if needed: `crates/agent-cli/src/repl.rs`
- Modify: `crates/agent-cli/tests/slash_commands.rs`
- Modify: `crates/agent-cli/tests/plain_repl.rs`

- [ ] **Step 1: Add CLI/plain REPL tests**

Cover:

- `/help` is catalog-derived and includes aliases/surfaces without hard-coded drift
- `/status` renders bounded provider/model/workspace/mode/connection state
- `/sessions`, `/trace`, and `/tasks` return deterministic guidance in plain REPL
- `/quit` and `/exit` share the same exit behavior
- unknown slash commands are bounded and do not submit prompts

- [ ] **Step 2: Implement CLI rendering**

Use bridge catalog/status helpers in `handle_slash_command`; keep prompt execution and runtime side effects unchanged.

- [ ] **Step 3: Run CLI tests**

Run:

```bash
cargo test -p agent-cli --test slash_commands
cargo test -p agent-cli --test plain_repl
```

Expected: CLI and plain REPL slash tests pass.

### Task 5: Add TUI Catalog Parsing and Status State

**Files:**
- Modify: `crates/agent-tui/src/slash.rs`
- Modify: `crates/agent-tui/src/app.rs`
- Create if needed: `crates/agent-tui/src/status_panel.rs`
- Modify: `crates/agent-tui/src/render.rs`
- Modify: `crates/agent-tui/src/lib.rs`
- Modify: `crates/agent-tui/tests/slash_commands.rs`
- Create if needed: `crates/agent-tui/tests/status_panel.rs`

- [ ] **Step 1: Add TUI tests**

Cover catalog-backed parsing, aliases, `/help`, `/status`, `/sessions`, `/trace`, `/tasks`, `/clear`, `/quit`, status panel rendering, and unknown command errors.

- [ ] **Step 2: Implement TUI status hub**

Add status panel state derived from app status and bridge status snapshots. Route `/status` to visible status output instead of a placeholder diagnostic.

- [ ] **Step 3: Run TUI tests**

Run:

```bash
cargo test -p agent-tui --test slash_commands
cargo test -p agent-tui --test status_panel
```

Expected: TUI slash/status tests pass.

### Task 6: Update Docs, Verify, and Commit

**Files:**
- Modify: `docs/plans/M59-02-slash-command-status-hub.plan.md`
- Modify: `docs/plans/M59-cli-tui-daily-driver.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-cli -p agent-ui-bridge -p agent-tui --all-targets -- -D warnings
cargo test -p agent-ui-bridge --test slash_catalog
cargo test -p agent-cli --test slash_commands
cargo test -p agent-cli --test plain_repl
cargo test -p agent-tui --test slash_commands
cargo test -p agent-tui --test status_panel
```

Expected: focused checks pass.

- [ ] **Step 2: Commit**

Run exact-path staging for M59-02 source, tests, and docs, then:

```bash
git commit -m "feat: add m59 slash command status hub"
```

Expected: commit contains only M59-02 source, tests, and documentation/status updates.

### Completion Checklist

- [ ] Slash command catalog contracts are typed, serde-friendly, deterministic, and tested.
- [ ] CLI/plain REPL help and status output are catalog/status backed.
- [ ] TUI slash parsing and status routing are catalog/status backed.
- [ ] Aliases resolve consistently across CLI/plain REPL and TUI.
- [ ] Unknown slash commands are bounded, safe, and do not submit prompts.
- [ ] Existing M59-01 session slash behavior remains intact.
- [ ] Focused fmt/clippy/tests pass.
