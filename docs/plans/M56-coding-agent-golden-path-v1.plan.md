# M56 Coding Agent Golden Path v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement one deterministic local coding-agent golden path from repository bootstrap through final review evidence.

**Architecture:** Runtime behavior belongs in `agent-core` and supporting runtime crates. `agent-cli`, `agent-tui`, and `agent-desktop` must consume shared runtime contracts/events through `agent-ui-bridge` instead of owning separate coding-agent logic.

**Tech Stack:** Rust 2024 edition, Tokio where async already exists, serde contracts, existing fake provider, built-in tools, policy/audit/replay crates, Cargo fmt, Cargo clippy, Cargo test, `cargo xtask ci`.

---

### Task 1: Confirm M56 Inputs

**Files:**
- Read: `docs/specs/M56-coding-agent-golden-path-v1.spec.md`
- Read: `docs/roadmap/MAINLINE_CAPABILITY_BASELINE.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/architecture/README.md`

- [ ] **Step 1: Review product gates**

Run:

```bash
sed -n '1,260p' docs/specs/M56-coding-agent-golden-path-v1.spec.md
sed -n '1,220p' docs/roadmap/PRODUCT_GAP_BACKLOG.md
```

Expected: M56-GP-01 through M56-GP-05 are the P0 implementation queue.

### Task 2: Implement M56 Slices in Order

**Files:**
- Execute: `docs/plans/M56-01-repo-session-bootstrap.plan.md`
- Create later: `docs/specs/M56-02-edit-diff-flow.spec.md`
- Create later: `docs/plans/M56-02-edit-diff-flow.plan.md`
- Create later: `docs/specs/M56-03-command-test-loop.spec.md`
- Create later: `docs/plans/M56-03-command-test-loop.plan.md`
- Create later: `docs/specs/M56-04-final-review-summary.spec.md`
- Create later: `docs/plans/M56-04-final-review-summary.plan.md`
- Create later: `docs/specs/M56-05-golden-path-e2e.spec.md`
- Create later: `docs/plans/M56-05-golden-path-e2e.plan.md`

- [ ] **Step 1: Complete M56-01 repo session bootstrap**

Open a workspace, load effective config, detect Git state, produce safe provider diagnostics, create durable ids, and expose CLI/JSON output.

- [ ] **Step 2: Create and complete M56-02 edit/diff flow**

Strengthen edit preview/apply, diff summary, approval payloads, and changed-file evidence.

- [ ] **Step 3: Create and complete M56-03 command/test loop**

Stream or bound command output, support cancellation-ready state, classify failures, and feed observations back to the model loop.

- [ ] **Step 4: Create and complete M56-04 final review summary**

Produce a final review contract with changed files, commands/tests, trace/audit evidence, unresolved risks, and next actions.

- [ ] **Step 5: Create and complete M56-05 golden path E2E**

Add deterministic fake-provider E2E/eval evidence and document real-provider opt-in behavior.

### Task 3: Verification Gates

**Files:**
- Read: `Cargo.toml`
- Read: `xtask/src/lib.rs`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [ ] **Step 1: Run focused tests after each slice**

Use the focused commands named by each slice plan.

- [ ] **Step 2: Run final M56 gate**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all gates pass and M56 evidence artifacts are redacted.

### Task 4: Complete M56

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Modify: `docs/plans/M56-coding-agent-golden-path-v1.plan.md`

- [ ] **Step 1: Update product docs**

Document the golden path command, fake-provider fixture, real-provider opt-in path, trace/audit/replay evidence, and known limitations.

- [ ] **Step 2: Commit completion**

Run exact-path staging for M56 files, source, tests, and docs, then:

```bash
git commit -m "feat: complete m56 coding agent golden path"
```

Expected: commit contains implementation, tests, and docs for M56 completion.

### Completion Checklist

- [ ] M56-01 through M56-05 are implemented and committed in focused stages.
- [ ] CLI/JSON can run the deterministic local coding golden path.
- [ ] Trace, permission, audit, replay, and final review evidence are present.
- [ ] TUI/Desktop bridge contracts remain shared-runtime based.
- [ ] Final M56 verification gates pass.
