# M66-05 CLI/Desktop Extension Surfaces and Support Evidence Plan

> **For agentic workers:** Support evidence is allowlisted metadata. Do not copy raw extension manifests, process logs, prompts, or OAuth material.

**Goal:** Add support-safe extension health evidence and polish the typed Desktop extension health surface.

**Architecture:** `xtask` owns support bundle manifests. Desktop renders `UiExtensionSnapshot` only.

---

### Task 1: Add Support Evidence Section

**Files:**
- Modify: `xtask/src/lib.rs`
- Modify: `xtask/tests/support_bundle.rs`
- Modify: `docs/release/support-bundle.md`

- [x] **Step 1: Add extension health section**

Add `extension_health_summary` as an included metadata-only support section.

- [x] **Step 2: Add redaction diagnostics**

Add diagnostics for raw extension manifests, raw extension logs, and raw OAuth material being omitted.

### Task 2: Polish Desktop Extension Health Surface

**Files:**
- Modify: `apps/desktop/src/components/ExtensionHealthView.tsx`
- Modify: `apps/desktop/src/__tests__/extension-health-ui.test.tsx`

- [x] **Step 1: Render severity and truncation**

Show diagnostic severity and truncation state from typed bridge snapshots.

- [x] **Step 2: Preserve redaction tests**

Ensure raw secret-like values remain absent from rendered output.

### Task 3: Update Status Docs

**Files:**
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark M66-05 complete after checks pass**

Update milestone status and make M66-06 the next closeout slice.

### Task 4: Verification

**Commands:**

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p xtask support_bundle
npm --prefix apps/desktop test -- extension-health-ui
npm --prefix apps/desktop run build
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 1: Run focused tests**

Run xtask support bundle tests and Desktop extension health UI tests.

- [x] **Step 2: Run build/format checks**

Run Desktop build, Rust fmt, and whitespace checks.

### Task 5: Commit

- [x] **Step 1: Commit M66-05**

Suggested message:

```text
feat: add extension support evidence
```
