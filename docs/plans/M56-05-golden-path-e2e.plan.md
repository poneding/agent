# M56-05 Golden Path E2E Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic M56 golden-path E2E/eval scenario that proves repo session, edit/diff, command/test, final review, trace/audit, replay, and CLI/JSON evidence under default CI-safe checks.

**Architecture:** Runtime behavior remains in `agent-core`; eval and CLI orchestrate only typed scenario execution and reporting. Evidence and goldens must be stable, bounded, and redacted.

**Tech Stack:** Rust 2024 edition, serde eval manifests, existing local coding fixture/final review contracts, `agent-eval`, `agent-cli`, `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-05-golden-path-e2e.spec.md`
- Read: `docs/eval/default.suite.json`
- Read: `docs/eval/scenarios/m04-replay.eval.json`
- Read: `docs/eval/scenarios/m04-evidence.eval.json`
- Read: `crates/agent-eval/src/manifest.rs`
- Read: `crates/agent-eval/src/runner.rs`
- Read: `crates/agent-core/src/local_coding.rs`
- Read: `crates/agent-core/src/final_review.rs`
- Read: `crates/agent-cli/src/command.rs`

- [ ] **Step 1: Review current eval behavior**

Run:

```bash
cargo test -p agent-cli eval_commands
cargo test -p agent-cli eval_suite_commands
```

Expected: current replay/evidence eval paths pass, while local-coding strict support is not yet a passing default suite scenario.

### Task 2: Write Failing Eval/CLI Tests

**Files:**
- Modify: `crates/agent-eval/tests/runner.rs`
- Modify: `crates/agent-eval/tests/corpus.rs`
- Create: `crates/agent-cli/tests/golden_path_e2e.rs`
- Modify if needed: `crates/agent-cli/tests/eval_suite_commands.rs`

- [ ] **Step 1: Add eval tests**

Cover:

- strict `local_coding` scenario passes with deterministic changed-file, command/test, final review, trace/audit, and replay evidence
- missing required local-coding evidence fails or becomes invalid in strict mode
- generated report is redacted and stable

- [ ] **Step 2: Add CLI smoke tests**

Cover:

- default eval suite includes the M56 scenario
- `cargo xtask eval` equivalent command path reports the M56 scenario as passed
- JSON output includes parseable report/golden evidence without secrets

- [ ] **Step 3: Run focused tests and verify failure**

Run:

```bash
cargo test -p agent-eval local_coding
cargo test -p agent-cli golden_path_e2e
```

Expected: tests fail before local-coding eval support and scenario files are implemented.

### Task 3: Implement Local Coding Eval Support

**Files:**
- Modify: `crates/agent-eval/src/runner.rs`
- Modify if needed: `crates/agent-eval/src/report.rs`
- Modify if needed: `crates/agent-eval/src/manifest.rs`

- [ ] **Step 1: Add local-coding runner path**

Use existing `agent-core` local coding/final review contracts to produce deterministic eval evidence.

- [ ] **Step 2: Validate expectations**

Check required events/evidence, final review fields, trace/audit/replay status, and strict failure behavior.

- [ ] **Step 3: Ensure redaction/stability**

Normalize temp paths, timestamps, IDs, and secret-like strings before golden comparison.

### Task 4: Add Scenario, Golden, and Suite Entry

**Files:**
- Create: `docs/eval/scenarios/m56-golden-path.eval.json`
- Create: `docs/eval/goldens/default-m56-golden-path.golden.json`
- Modify: `docs/eval/default.suite.json`

- [ ] **Step 1: Add scenario manifest**

Create the M56 local-coding scenario with strict expectations.

- [ ] **Step 2: Generate or hand-author stable golden**

Use the existing golden format and ensure evidence is bounded/redacted.

- [ ] **Step 3: Add scenario to default suite**

Update default suite and CLI tests to expect three scenarios.

### Task 5: Verify and Commit

**Files:**
- Modify: `docs/plans/M56-05-golden-path-e2e.plan.md`
- Modify: `docs/milestones/README.md`
- Modify if needed: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [ ] **Step 1: Run focused verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-core -p agent-cli -p agent-eval --all-targets -- -D warnings
cargo test -p agent-eval local_coding
cargo test -p agent-cli golden_path_e2e
cargo test -p agent-cli eval_suite_commands
cargo xtask eval
```

Expected: focused checks and default eval suite pass.

- [ ] **Step 2: Commit**

Run exact-path staging for M56-05 source, tests, eval artifacts, and docs, then:

```bash
git commit -m "feat: add m56 golden path e2e"
```

Expected: commit contains only M56-05 source, tests, eval scenario/golden, and documentation/status updates.

### Completion Checklist

- [ ] Default eval suite includes M56 golden path.
- [ ] Local-coding eval scenario passes strict mode.
- [ ] CLI/JSON evidence is parseable and redacted.
- [ ] Golden evidence is deterministic.
- [ ] Trace/audit/replay/final review evidence is present.
- [ ] Focused fmt/clippy/tests and `cargo xtask eval` pass.
