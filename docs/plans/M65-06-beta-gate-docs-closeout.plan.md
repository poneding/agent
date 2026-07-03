# M65-06 Beta Gate Docs and Closeout Plan

> **For agentic workers:** Closeout means evidence, not optimism. Mark M65 complete only after the final dev-channel gate passes and the status docs point to M66.

**Goal:** Document the beta install/upgrade/support workflows, run the final gate, and close M65 with M66 as the next active milestone.

**Architecture:** No new runtime architecture is expected. This slice ties together existing `xtask`, runtime diagnostics, CLI/Desktop docs, and release evidence.

**Tech Stack:** Markdown docs, Rust workspace gates, Desktop npm gates, `xtask` release commands.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-06-beta-gate-docs-closeout.spec.md`
- Read: `docs/release/beta-gate.md`
- Read: `docs/release/support-bundle.md`
- Read: `docs/cli/release-packaging.md`
- Read: `docs/desktop/release-packaging.md`
- Read: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Read: `docs/milestones/README.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Verify M65-01 through M65-05 status**

M65-01 through M65-05 are complete. M65-06 is the active closeout slice.

### Task 2: Add Closeout Workflow Docs

**Files:**
- Create: `docs/release/install-upgrade-support.md`
- Modify: `docs/release/beta-gate.md`
- Modify: `docs/release/support-bundle.md`
- Modify: `README.md`

- [x] **Step 1: Document beta install workflow**

Explain CLI package and Desktop package/update commands, dev-channel plan behavior, artifact paths, and smoke roots.

- [x] **Step 2: Document upgrade/onboarding workflow**

Explain config diagnostics, onboarding readiness statuses, settings/secrets/migration/provider setup, and failure recovery.

- [x] **Step 3: Document support workflow**

Explain support bundle command, manifest/summary output, allowlisted evidence, and forbidden content.

### Task 3: Run Final M65 Gate

**Files:**
- Generated evidence under `target/`

- [x] **Step 1: Run final dev-channel commands**

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m65-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m65-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask package-cli --channel dev
cargo xtask package-desktop --channel dev
cargo xtask support-bundle --channel dev
git diff --check
```

Expected: all commands pass and release/support evidence is generated.

### Task 4: Close M65 Status

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Modify: `docs/plans/M65-06-beta-gate-docs-closeout.plan.md`

- [x] **Step 1: Mark M65-06 complete**

Update slice index and M65 milestone plan after final gates pass.

- [x] **Step 2: Mark M65 complete and M66 next active**

Status docs should make M66 Extensions and MCP Hardening the next active milestone.

### Task 5: Commit

**Files:**
- M65-06 spec/plan, release docs, README, status docs.

- [x] **Step 1: Commit closeout**

Suggested commit message:

```text
docs: close m65 beta gate
```
