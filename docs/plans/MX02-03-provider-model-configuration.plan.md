# MX02-03 Provider and Model Configuration Implementation Plan

> **For agentic workers:** If the `superpowers` skill is available, use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to execute this plan task by task. If it is unavailable, follow the same strict checklist directly.

**Goal:** Add provider/model profiles and Desktop settings/composer controls that are capability-aware and secret-safe.

**Architecture:** `ruse-model` validates provider/model profiles, `ruse-ui-bridge` serializes UI-safe snapshots, `ruse-desktop` builds settings view models, Tauri validates/redacts, and React renders controls.

**Tech Stack:** Rust serde contracts, Tauri command tests, React/TypeScript, Vitest.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/MX02-03-provider-model-configuration.spec.md`
- Read: `crates/ruse-model/src/config.rs`
- Read: `crates/ruse-desktop/src/settings.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `apps/desktop/src/components/SettingsView.tsx`
- Read: `apps/desktop/src/state/settings.ts`
- Read: `apps/desktop/src/types.ts`
- Read: `apps/desktop/src-tauri/src/settings_commands.rs`

- [ ] **Step 1: Review current config shape**

Expected: identify existing provider id/kind/default model/endpoint/secret ref/capability snapshot behavior.

### Task 2: Extend Contracts

**Files:**
- Modify: `crates/ruse-model/src/config.rs`
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-desktop/src/settings.rs`
- Modify: focused Rust tests

- [ ] **Step 1: Add provider/model profiles**

Add typed profile structures with diagnostics, capabilities, model list, defaults, and secret ref status.

- [ ] **Step 2: Add validation/redaction tests**

Prove secrets never appear in snapshots and invalid providers produce diagnostics.

### Task 3: Implement Settings and Composer UI

**Files:**
- Modify: `apps/desktop/src/components/SettingsView.tsx`
- Modify: `apps/desktop/src/components/Composer.tsx`
- Modify: `apps/desktop/src/state/settings.ts`
- Modify: `apps/desktop/src/types.ts`
- Modify: `apps/desktop/src-tauri/src/settings_commands.rs`
- Modify: focused frontend/Tauri tests

- [ ] **Step 1: Add provider management UI**

Add provider list, model cards/table, add/edit dialog, base URL, secret status, default model selector, and diagnostics.

- [ ] **Step 2: Add composer integration**

Expose provider/model selector and capability hints in the composer with per-thread override state.

### Task 4: Verify

- [ ] **Step 1: Run focused checks**

Run:

```bash
cargo clippy -p ruse-model -p ruse-desktop -p ruse-ui-bridge --all-targets -- -D warnings
cargo test -p ruse-model
cargo test -p ruse-desktop --test settings
cargo test -p ruse-ui-bridge --test config_snapshot
npm --prefix apps/desktop test -- trace-settings-ui
npm --prefix apps/desktop run build
```

Expected: all focused provider/model checks pass.

### Task 5: Commit

- [ ] **Step 1: Commit slice**

Run:

```bash
git diff --check
git add crates/ruse-model crates/ruse-desktop crates/ruse-ui-bridge apps/desktop docs/specs/MX02-03-provider-model-configuration.spec.md docs/plans/MX02-03-provider-model-configuration.plan.md
git commit -m "feat: productize desktop provider settings"
```

Expected: commit succeeds with secret-safe provider/model contracts.
