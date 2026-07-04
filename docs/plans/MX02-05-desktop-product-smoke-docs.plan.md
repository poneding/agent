# MX02-05 Desktop Product Smoke and Documentation Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly.

**Goal:** Prove and document the productized Desktop workflow with deterministic, credential-free smoke coverage.

**Architecture:** Smoke consumes typed Desktop/bridge fixtures and React rendering. Runtime truth remains in Rust contracts. Docs describe usage and verification commands.

**Tech Stack:** Vitest, React Testing Library, Vite build, existing `cargo xtask desktop-test`, Rust workspace gates.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/MX02-05-desktop-product-smoke-docs.spec.md`
- Read: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Read: `docs/desktop/daily-driver-smoke.md`
- Read: `README.md`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/milestones/README.md`

- [ ] **Step 1: Review current Desktop smoke**

Expected: identify current M60 smoke coverage and gaps for productized shell/thread/composer/inspector/settings flow.

### Task 2: Add Product Smoke

**Files:**
- Modify: `apps/desktop/src/__tests__/desktop-smoke.test.tsx`
- Modify: `xtask/` only if `cargo xtask desktop-test` needs explicit updates

- [ ] **Step 1: Build deterministic fixture**

Cover open workspace, create thread, choose provider/model, submit prompt, receive typed frames, show approval, show diff, show task progress, open settings, and verify no secret leakage.

- [ ] **Step 2: Connect smoke command**

Update `cargo xtask desktop-test` if required. Otherwise document that existing `cargo xtask desktop-test` includes the Vitest smoke.

### Task 3: Write Documentation

**Files:**
- Create: `docs/desktop/desktop-productization.md`
- Modify: `README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/milestones/README.md`

- [ ] **Step 1: Document Desktop workflow**

Describe the productized shell, thread navigation, composer, command palette, inspector, settings, terminal drawer placeholder, and verification commands.

- [ ] **Step 2: Close out status docs**

Update MX02 status and slice completion evidence after implementation gates pass.

### Task 4: Run Final Gates

- [ ] **Step 1: Run final checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask desktop-test
cargo xtask eval
```

Expected: all MX02 final gates pass.

### Task 5: Commit

- [ ] **Step 1: Commit closeout**

Run:

```bash
git diff --check
git add README.md docs/desktop/desktop-productization.md docs/roadmap/PRODUCT_ROADMAP.md docs/milestones/README.md apps/desktop xtask docs/specs/MX02-05-desktop-product-smoke-docs.spec.md docs/plans/MX02-05-desktop-product-smoke-docs.plan.md
git commit -m "test: add desktop product smoke"
```

Expected: closeout commit includes smoke, docs, status updates, and no unrelated recovery files.
