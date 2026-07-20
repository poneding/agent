# M65-04 Settings, Secrets, and Migration Onboarding Plan

> **For agentic workers:** Runtime crates own onboarding readiness. CLI, TUI, Desktop, and support bundle code should render or collect typed snapshots, not reimplement settings, secret, migration, or provider policy.

**Goal:** Add beta-grade onboarding readiness diagnostics for settings, paths, secret store health, startup migrations, and provider/model readiness.

**Architecture:** `ruse-core` builds the readiness snapshot; `ruse-ui-bridge` maps it into a UI-safe frame; CLI/TUI render bounded summaries.

**Tech Stack:** Rust 2024 edition, serde contracts, existing config/settings/secrets/migration/provider support crates, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M65-04-settings-secrets-migration-onboarding.spec.md`
- Read: `crates/ruse-core/src/startup_migration.rs`
- Read: `crates/ruse-core/src/provider_support.rs`
- Read: `crates/ruse-model/src/secrets.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `crates/ruse-ui-bridge/src/frame.rs`

- [x] **Step 1: Review existing settings/secrets/migration/provider contracts**

Existing contracts already cover effective runtime config, provider support, platform paths/settings profile, startup migration outcomes, and secret store health.

### Task 2: Add Onboarding Readiness Contract

**Files:**
- Create: `crates/ruse-core/src/onboarding.rs`
- Modify: `crates/ruse-core/src/lib.rs`

- [x] **Step 1: Add typed readiness snapshot**

Add `OnboardingReadinessSnapshot` plus nested settings, path, secret, migration, provider, diagnostic, and next-action summaries.

- [x] **Step 2: Derive readiness from existing contracts**

Use `EffectiveRuntimeConfig`, `ProviderSupportReport`, `SettingsProfile`, `PlatformPaths`, `SecretStoreHealth`, and `StartupMigrationOutcome`.

- [x] **Step 3: Add redaction and status tests**

Cover ready baseline, degraded keychain/provider readiness, and blocked migration diagnostics without leaking secret-like values.

### Task 3: Add UI Bridge And TUI Rendering

**Files:**
- Modify: `crates/ruse-ui-bridge/src/frame.rs`
- Modify: `crates/ruse-ui-bridge/src/lib.rs`
- Modify: `crates/ruse-tui/src/trace_panel.rs`
- Modify: `crates/ruse-tui/src/transcript.rs`

- [x] **Step 1: Add additive bridge frame**

Add `UiOnboardingReadinessSnapshot` and `UiJsonFrame::OnboardingReadinessSnapshot`.

- [x] **Step 2: Preserve stable diagnostic codes**

Keep machine-stable ids such as `secret_store.keychain_unavailable` intact while redacting messages.

- [x] **Step 3: Render summary in TUI**

TUI trace/transcript displays status, provider/model, secret-store availability, migration status, and diagnostic count.

### Task 4: Add CLI Config Check Output

**Files:**
- Modify: `crates/ruse-cli/src/command.rs`
- Modify: `crates/ruse-cli/tests/config_diagnostics.rs`

- [x] **Step 1: Build readiness snapshot during config check**

Run default startup migration checks, inspect keychain/env fallback health, and combine provider support with settings/platform paths.

- [x] **Step 2: Add text output**

Include onboarding readiness status, diagnostic count, and next-action count.

- [x] **Step 3: Add JSON output**

Emit the new onboarding frame after the existing config snapshot.

- [x] **Step 4: Extend tests**

Assert JSON/text output and redaction of secret-like config data.

### Task 5: Update Docs And Status

**Files:**
- Modify: `docs/release/beta-gate.md`
- Modify: `docs/release/support-bundle.md`
- Modify: `docs/plans/M65-packaging-release-beta-gate.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`

- [x] **Step 1: Document beta gate readiness evidence**

Explain that M65-04 supplies settings/secrets/migration/provider onboarding evidence.

- [x] **Step 2: Document support bundle input**

M65-05 should collect this snapshot rather than raw config, raw env, raw secrets, or raw logs.

- [x] **Step 3: Update status**

Mark M65-04 complete after focused gates pass and set M65-05 as next active slice.

### Task 6: Verify

- [x] **Step 1: Run focused tests**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo test -p ruse-core onboarding
CARGO_TARGET_DIR=target/m65-dev cargo test -p ruse-cli --test config_diagnostics
CARGO_TARGET_DIR=target/m65-dev cargo test -p ruse-ui-bridge -p ruse-tui --lib
```

- [x] **Step 2: Run final focused gates**

```bash
CARGO_TARGET_DIR=target/m65-dev cargo clippy -p ruse-core -p ruse-ui-bridge -p ruse-cli -p ruse-tui --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Task 7: Commit

**Files:**
- M65-04 spec/plan, onboarding core contract, bridge/TUI/CLI rendering, focused tests, release docs, milestone/roadmap status.

- [ ] **Step 1: Commit implementation**

Suggested commit message:

```text
feat: add m65 settings secrets migration onboarding
```
