# M56-06 Hardening Docs and Verification Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close M56 with user-facing golden-path documentation, final milestone status updates, and workspace-wide verification evidence.

**Architecture:** No runtime changes. Docs describe the existing `agent-cli` -> `agent-eval` -> `agent-core` path and keep eval artifacts redacted under `target/agent-eval`.

**Tech Stack:** Markdown docs, Cargo fmt, workspace clippy, workspace tests, `cargo xtask eval`.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M56-06-hardening-docs-verification.spec.md`
- Read: `README.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `docs/milestones/README.md`
- Read: `docs/plans/M56-coding-agent-golden-path-v1.plan.md`

- [ ] **Step 1: Review closeout scope**

Run:

```bash
sed -n '1,220p' docs/specs/M56-06-hardening-docs-verification.spec.md
```

Expected: closeout is docs/status/verification only, with no new runtime contracts.

### Task 2: Update Product Docs

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`

- [ ] **Step 1: Document the M56 acceptance command**

Add the deterministic default command and artifact paths:

```bash
cargo xtask eval
```

- [ ] **Step 2: Document real-provider limits**

State that default M56 is fixture-driven and credential-free; real-provider parity moves to M57.

### Task 3: Update Milestone Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M56-coding-agent-golden-path-v1.plan.md`
- Modify: `docs/plans/M56-06-hardening-docs-verification.plan.md`

- [ ] **Step 1: Mark M56 complete after gates pass**

Update milestone status, active focus, next action, and M57 handoff only after final verification succeeds.

### Task 4: Run Final Verification

**Files:**
- Read: `Cargo.toml`
- Read: `xtask/src/lib.rs`

- [ ] **Step 1: Run final M56 gates**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask eval
```

Expected: all commands pass and generated artifacts stay under ignored `target/` paths.

### Task 5: Commit

**Files:**
- Modify/Create only M56 closeout docs and status files.

- [ ] **Step 1: Commit closeout**

Run exact-path staging, then:

```bash
git commit -m "docs: complete m56 golden path"
```

Expected: commit contains M56-06 spec/plan, README/roadmap/milestone status updates, and no generated target artifacts.

### Completion Checklist

- [ ] README documents the M56 golden-path command and artifacts.
- [ ] Roadmap marks M56 complete and M57 next.
- [ ] Milestones mark M56 implementation complete.
- [ ] Final fmt/clippy/test/eval gates pass.
- [ ] M56 closeout commit is created.
