# M25-00 Packaging and Release Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M25 architecture baseline for packaging, settings, secrets, migrations, release artifacts, and updates.

**Architecture:** This is a documentation-only slice. It defines shared platform paths, settings mutation, OS keychain and env secret backends, startup migrations, CLI/Desktop packaging, release channels, and smoke tests.

**Tech Stack:** Markdown docs, packaging/release design notes, existing config/storage/secret architecture docs, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M25-packaging-release-settings-updates.spec.md`
- Read: `docs/architecture/configuration.md`
- Read: `docs/architecture/storage.md`
- Read: `docs/specs/M14-00-runtime-configuration-secrets-technical-design.spec.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M25-packaging-release-settings-updates.spec.md
```

Expected: milestone scope defines packaging, paths, settings, secrets, migrations, and updates.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M25-00-packaging-release-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document platform paths, settings model, secret stores, migration flow, release artifacts, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that raw secrets never enter config files, migrations run before side effects, and update failures leave the current version usable.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M25-00-packaging-release-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M25-00-packaging-release-technical-design.spec.md`
- Add: `docs/plans/M25-00-packaging-release-technical-design.plan.md`
- Modify: `docs/plans/M25-packaging-release-settings-updates.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M25-00-packaging-release-technical-design.spec.md docs/plans/M25-00-packaging-release-technical-design.plan.md docs/plans/M25-packaging-release-settings-updates.plan.md
git commit -m "docs: add m25 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
