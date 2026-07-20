# M63-05 Team Workflow Smoke, Docs, and Closeout Plan

> **For agentic workers:** Keep this slice focused on smoke evidence, docs, and M63 closeout. Do not start M64 memory work or Desktop redesign in this slice.

**Goal:** Add deterministic team workflow smoke coverage, document CLI/Desktop team workflows, run final gates, and close M63.

**Architecture:** `ruse-team` owns workflow and merge-readiness contracts. `ruse-eval` runs a credential-free smoke over typed snapshots and reports stable golden evidence. CLI/Desktop docs describe existing bridge/rendering surfaces without moving policy or readiness logic into presentation layers.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M63-worktree-ruse-team-workflows.spec.md`
- Read: `docs/plans/M63-worktree-ruse-team-workflows.plan.md`
- Read: `docs/specs/M63-01-team-workflow-snapshot-readiness.spec.md`
- Read: `docs/specs/M63-02-cli-json-team-workflows.spec.md`
- Read: `docs/specs/M63-03-reviewer-merge-readiness.spec.md`
- Read: `docs/specs/M63-04-tui-desktop-team-surfaces.spec.md`
- Read: `crates/ruse-team/`
- Read: `crates/ruse-eval/`
- Read: `crates/ruse-cli/tests/team_workflow_commands.rs`
- Read: `docs/eval/default.suite.json`

- [x] **Step 1: Review M63 contracts**

Identify the existing typed evidence needed for closeout: worktrees, handoffs, blackboard entries, review votes, capability ceilings, conflicts, diagnostics, pending approvals, and merge blockers.

### Task 2: Add Team Workflow Eval Smoke

**Files:**
- Modify: `crates/ruse-eval/Cargo.toml`
- Modify: `crates/ruse-eval/src/manifest.rs`
- Modify: `crates/ruse-eval/src/report.rs`
- Modify: `crates/ruse-eval/src/runner.rs`
- Modify: `crates/ruse-eval/tests/runner.rs`
- Modify: `crates/ruse-eval/tests/manifest.rs`
- Modify: `crates/ruse-eval/tests/corpus.rs`
- Modify: `docs/eval/default.suite.json`
- Create: `docs/eval/scenarios/m63-team-workflow-smoke.eval.json`
- Create: `docs/eval/goldens/default-m63-team-workflow-smoke.golden.json`

- [x] **Step 1: Add eval kind**

Add `team_workflow_smoke` as a fixture-free eval scenario kind with stable manifest parsing and reporting.

- [x] **Step 2: Build deterministic smoke**

Construct typed team workflow snapshots that prove:

- isolated worktree evidence
- reduced-capability handoff evidence
- blackboard output
- reviewer vote evidence
- conflict-aware merge readiness
- policy-blocked capability ceiling
- redaction of secret-like strings

- [x] **Step 3: Add golden and default suite entry**

Add the scenario to `docs/eval/default.suite.json` and commit a stable normalized golden.

- [x] **Step 4: Run eval-focused checks**

```bash
cargo test -p ruse-eval
cargo test -p ruse-cli --test eval_suite_commands
cargo xtask eval
```

Result: `cargo test -p ruse-eval`, `cargo test -p ruse-cli --test eval_suite_commands`, and `cargo xtask eval` pass. The default eval suite reports `total=6 passed=6 failed=0 skipped=0 invalid=0 not_run=0`, including `default-m63-team-workflow-smoke`.

### Task 3: Add Team Workflow Docs

**Files:**
- Create: `docs/cli/team-workflows.md`
- Create: `docs/desktop/team-workflows.md`
- Modify: `README.md`

- [x] **Step 1: Document CLI workflows**

Cover team status, handoff dry-run, blackboard inspect, review dry-run, merge-readiness dry-run, fixtures, readiness states, redaction, and dry-run limits.

- [x] **Step 2: Document Desktop workflows**

Cover the Tasks-route team workflow panel, readiness, handoffs, blackboard, merge review, capability ceiling, conflicts, diagnostics, and how Desktop consumes bridge snapshots.

- [x] **Step 3: Link docs**

Add README links or milestone references where appropriate.

### Task 4: Close M63 Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M63-worktree-ruse-team-workflows.plan.md`
- Modify: `docs/plans/M63-05-team-workflow-smoke-closeout.plan.md`

- [x] **Step 1: Run final closeout gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
git diff --check
```

Result: Final closeout gates passed with `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `npm --prefix apps/desktop test`, `npm --prefix apps/desktop run build`, `cargo xtask eval`, and `git diff --check`. The workspace test output includes an expected negative-path `definitely-not-a-real-rustc-flag` message from the xtask quality-gate tests while the command exits successfully. `cargo xtask eval` reports `total=6 passed=6 failed=0 skipped=0 invalid=0 not_run=0`.

- [x] **Step 2: Update status docs**

Mark:

- M63-05 spec, plan, and implementation complete
- M63 implementation complete
- roadmap M63 status complete
- next active focus M64 Memory and Personalization

Do not mark M64-M66 complete.

- [ ] **Step 3: Commit M63-05**

Exact-stage only M63-05 source, eval, docs, and status files. Exclude `tmp/`.

Suggested commit message:

```text
feat: add m63 team workflow smoke closeout
```
