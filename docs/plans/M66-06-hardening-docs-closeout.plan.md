# M66-06 Hardening Docs and Closeout Plan

> **For agentic workers:** M66-06 is a closeout slice. Keep changes to documentation, status updates, and verification unless a final gate exposes an M66-owned defect.

**Goal:** Document extension management workflows, align roadmap/status evidence, run final gates, and close M66.

**Architecture:** Runtime crates and `xtask` own extension/support behavior. CLI/Desktop render typed contracts. Documentation should describe that boundary rather than adding presentation-local logic.

---

### Task 1: Document Extension Workflows

**Files:**
- Create: `docs/extensions/management-health.md`
- Modify: `README.md`
- Modify if needed: `docs/release/support-bundle.md`

- [x] **Step 1: Document plugin workflows**

Describe install, list, inspect, enable, disable, remove, compatibility blocking, capability acceptance, and lease evidence.

- [x] **Step 2: Document skill root workflows**

Describe add, list, inspect, enable, disable, remove, registry refresh, and duplicate diagnostics.

- [x] **Step 3: Document MCP health and OAuth expectations**

Describe MCP status health diagnostics, exported capability mismatch checks, and OAuth deferral boundaries.

- [x] **Step 4: Document support evidence boundaries**

Describe `extension_health_summary` and forbidden raw extension/OAuth content.

### Task 2: Update Status Indexes

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Modify: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark M66-06 complete**

Update milestone slice index and planning completeness.

- [x] **Step 2: Mark M66 complete**

Update roadmap, backlog, and smoke matrix only after workflow docs exist.

### Task 3: Verification

**Commands:**

```bash
cargo fmt --all -- --check
CARGO_TARGET_DIR=target/m66-dev cargo clippy --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=target/m66-dev cargo test --workspace
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo xtask eval
cargo xtask support-bundle --channel dev
git diff --check
```

- [x] **Step 1: Run final M66 gates**

Run the closeout gates and document any unrelated failure with focused passing evidence.

### Task 4: Commit

- [ ] **Step 1: Commit M66 closeout**

Suggested message:

```text
docs: close m66 extension hardening
```
