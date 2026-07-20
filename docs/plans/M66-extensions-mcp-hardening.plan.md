# M66 Extensions and MCP Hardening Plan

> **For agentic workers:** M66 is product hardening, not protocol wandering. Only add MCP/OAuth work that improves installable plugin, skill, or MCP server workflows.

**Goal:** Make extensions manageable, diagnosable, compatible, and supportable after M65 beta paths are credible.

**Architecture:** Runtime/extension crates own compatibility, health, capability review, OAuth state, and support evidence. CLI/TUI/Desktop render typed summaries.

**Tech Stack:** Rust 2024 edition, existing `ruse-plugin`, `ruse-skill`, `ruse-mcp`, `ruse-core`, `ruse-ui-bridge`, `ruse-cli`, Desktop React/Vitest when touched, Cargo fmt/clippy/test.

---

### Task 1: Confirm Baseline

**Files:**
- Read: `docs/specs/M66-extensions-mcp-hardening.spec.md`
- Read: `docs/roadmap/PRODUCT_GAP_BACKLOG.md`
- Read: `docs/roadmap/GOLDEN_PATH_SMOKE_MATRIX.md`
- Read: `crates/ruse-core/src/extension.rs`
- Read: `crates/ruse-plugin/`
- Read: `crates/ruse-skill/`
- Read: `crates/ruse-mcp/`
- Read: `crates/ruse-ui-bridge/src/frame.rs`
- Read: `crates/ruse-cli/src/command.rs`
- Read: `apps/desktop/src/components/ExtensionHealthView.tsx`

- [x] **Step 1: Confirm current extension foundations**

Existing foundations include plugin install/manage commands, skill root commands, extension registry snapshots, MCP management snapshots, CLI MCP status/profile/export commands, TUI extension panels, Desktop extension health rendering, and MX01 MCP epilogue hardening.

### Task 2: M66-01 Extension Health Baseline and Smoke

**Files:**
- Create: `docs/specs/M66-01-extension-health-baseline-smoke.spec.md`
- Create: `docs/plans/M66-01-extension-health-baseline-smoke.plan.md`
- Modify: `crates/ruse-core/src/extension.rs`
- Modify if needed: `crates/ruse-ui-bridge/`
- Modify if needed: `crates/ruse-cli/`
- Modify if needed: `crates/ruse-eval/`
- Modify: `docs/`

- [x] **Step 1: Plan extension health smoke**

Define the typed evidence proving plugin, skill, and MCP health summaries are bounded, redacted, deterministic, and support-safe.

- [x] **Step 2: Implement extension health smoke**

Reuse existing extension registry, plugin, skill, and MCP contracts. Do not create a parallel extension system.

### Task 3: M66-02 Plugin and Skill Management Hardening

**Files:**
- Create: `docs/specs/M66-02-plugin-skill-management-hardening.spec.md`
- Create: `docs/plans/M66-02-plugin-skill-management-hardening.plan.md`
- Modify as needed: `crates/ruse-plugin/`, `crates/ruse-skill/`, `crates/ruse-cli/`, `crates/ruse-ui-bridge/`

- [x] **Step 1: Plan plugin/skill hardening**

Focus on install/enable/disable/remove, capability acceptance, compatibility, redaction, and stable CLI/JSON outputs.

- [x] **Step 2: Implement plugin/skill hardening**

Add only user-visible management polish and tests.

### Task 4: M66-03 MCP Management and Compatibility Health

**Files:**
- Create: `docs/specs/M66-03-mcp-management-compatibility-health.spec.md`
- Create: `docs/plans/M66-03-mcp-management-compatibility-health.plan.md`
- Modify as needed: `crates/ruse-mcp/`, `crates/ruse-core/`, `crates/ruse-ui-bridge/`, `crates/ruse-cli/`

- [x] **Step 1: Plan MCP management health**

Define compatibility, server status, exported capabilities, roots/auth/stream state, and diagnostics from existing contracts.

- [x] **Step 2: Implement MCP management health**

Keep evidence bounded and redacted. Avoid broad protocol expansion.

### Task 5: M66-04 OAuth Completion for Installable MCP Workflows

**Files:**
- Create: `docs/specs/M66-04-mcp-oauth-installable-workflows.spec.md`
- Create: `docs/plans/M66-04-mcp-oauth-installable-workflows.plan.md`
- Modify as needed only if installable workflows require it.

- [x] **Step 1: Confirm OAuth need**

If no user-installable workflow requires OAuth completion, document deferral and do not implement extra protocol code.

- [x] **Step 2: Implement only required OAuth completion**

Keep token values out of durable evidence and support bundles.

### Task 6: M66-05 CLI/Desktop Extension Surfaces and Support Evidence

**Files:**
- Create: `docs/specs/M66-05-cli-desktop-extension-support.spec.md`
- Create: `docs/plans/M66-05-cli-desktop-extension-support.plan.md`
- Modify as needed: CLI, TUI, Desktop, bridge, support bundle docs/code.

- [x] **Step 1: Plan surface/support updates**

Render shared snapshots and include extension evidence in support bundles without raw logs/manifests/secrets.

- [x] **Step 2: Implement surface/support updates**

Presentation layers render typed contracts only.

### Task 7: M66-06 Hardening Docs and Closeout

**Files:**
- Create: `docs/specs/M66-06-hardening-docs-closeout.spec.md`
- Create: `docs/plans/M66-06-hardening-docs-closeout.plan.md`
- Modify: README, roadmap, milestone docs, extension docs.

- [x] **Step 1: Document extension workflows**

Document plugin, skill, MCP management, health, compatibility, support, and OAuth expectations.

- [x] **Step 2: Run final M66 gates**

Run repo standard gates plus focused extension/MCP/support checks.

### Task 8: Commit Discipline

**Files:**
- Modify/create only files for the active slice and required status docs.

- [x] **Step 1: Commit M66 planning**

Suggested commit message:

```text
docs: add m66 extensions hardening plan
```

- [x] **Step 2: Commit each implementation slice**

Exact-stage slice files, code, tests, docs, and status updates only.
