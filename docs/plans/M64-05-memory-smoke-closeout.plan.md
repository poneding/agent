# M64-05 Memory Smoke, Docs, and Closeout Plan

> **For agentic workers:** Keep smoke logic deterministic and credential-free. The eval should exercise memory contracts through crates, not through provider calls or Desktop-only fixtures.

**Goal:** Add default eval coverage and user docs that prove M64 memory is useful, reviewable, privacy-safe, and visible across CLI/TUI/Desktop.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M64-memory-personalization.spec.md`
- Read: `docs/specs/M64-05-memory-smoke-closeout.spec.md`
- Read: `docs/eval/default.suite.json`
- Read: `crates/agent-eval/src/manifest.rs`
- Read: `crates/agent-eval/src/runner.rs`
- Read: `docs/cli/memory-review.md`
- Read: `apps/desktop/src/components/MemoryView.tsx`

- [x] **Step 1: Confirm M64 slice evidence**

M64-01 through M64-04 are complete. M64-05 should close the milestone; it should not add new memory policy or broad UI redesign.

### Task 2: Add Deterministic Memory Smoke Eval

**Files:**
- Modify: `crates/agent-eval/Cargo.toml`
- Modify: `crates/agent-eval/src/manifest.rs`
- Modify: `crates/agent-eval/src/runner.rs`
- Modify: `crates/agent-eval/src/report.rs`
- Modify: `crates/agent-eval/src/lib.rs`
- Modify: `crates/agent-eval/tests/`
- Create: `docs/eval/scenarios/m64-memory-smoke.eval.json`
- Create: `docs/eval/goldens/default-m64-memory-smoke.golden.json`
- Modify: `docs/eval/default.suite.json`

- [ ] **Step 1: Add eval manifest contracts**

Add a `memory_smoke` scenario kind and `EvalMemoryExpectations` for minimum record/proposal/retrieval/tombstone evidence and required booleans for import/export, rebuild, disabled-memory diagnostics, and redaction.

- [ ] **Step 2: Add deterministic runner**

The runner should construct memory records, retrieval candidates, runtime memory evidence, proposal decisions, import/export, tombstones, rebuild evidence, and disabled-memory evidence without credentials.

- [ ] **Step 3: Add default suite scenario and golden**

Add `default-m64-memory-smoke` to `docs/eval/default.suite.json` and update corpus tests/golden.

### Task 3: Add User-Facing Docs

**Files:**
- Modify: `docs/cli/memory-review.md`
- Create: `docs/desktop/memory-workflows.md`
- Modify if needed: `docs/desktop/daily-driver-smoke.md`

- [ ] **Step 1: Document Desktop memory surface**

Describe the Tasks-route memory panel, records, proposals, retrieval/influence counts, diagnostics, retention/tombstone state, disabled-memory behavior, and redaction expectations.

- [ ] **Step 2: Close CLI memory doc gaps**

Ensure CLI docs mention the default eval smoke, disabled-memory behavior, import/export/tombstone/rebuild evidence, and proposal content redaction.

### Task 4: Verify and Close M64

**Files:**
- Modify: `docs/plans/M64-memory-personalization.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [ ] **Step 1: Run focused gates**

```bash
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test runner memory
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test manifest memory
CARGO_TARGET_DIR=target/m64-dev cargo test -p agent-eval --test corpus
cargo xtask eval
```

- [ ] **Step 2: Run final M64 gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
git diff --check
```

- [ ] **Step 3: Update status docs**

Mark M64 complete only after the final gates pass. Set M65 Packaging, Release, and Beta Gate as the next active milestone.

### Task 5: Commit

**Files:**
- Modify/create only M64-05 eval/docs/status files and required lockfile/dependency updates.

- [ ] **Step 1: Commit planning**

Suggested commit message:

```text
docs: add m64 memory smoke closeout plan
```

- [ ] **Step 2: Commit implementation**

Suggested commit message:

```text
feat: close m64 memory personalization
```
