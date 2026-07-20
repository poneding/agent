# M13-00 Real Model Providers And Runtime Configuration Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M13 technical design baseline for provider configuration, registry, invocation policy, OpenAI-compatible adapter, and CLI/Desktop selection boundaries.

**Architecture:** This is a documentation-only design slice. It constrains later source changes to `ruse-model`, `ruse-core`, `ruse-cli`, `ruse-desktop`, and storage/config contracts without creating runtime code.

**Tech Stack:** Markdown specs/plans, Mermaid diagrams, existing architecture docs, Cargo workspace gates.

---

### Task 1: Review Inputs

**Files:**
- Read: `docs/PRD.md`
- Read: `docs/architecture/configuration.md`
- Read: `docs/architecture/security-policy.md`
- Read: `docs/architecture/async-runtime.md`
- Read: `docs/architecture/error-model.md`

- [x] **Step 1: Confirm real-provider gap**

Run:

```bash
rg -n "real model provider|provider-agnostic|Secrets|model" docs/PRD.md docs/architecture
```

Expected: PRD and architecture require provider-agnostic model integration and secret-safe config.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`

- [x] **Step 1: Define M13 design**

Write the design sections for provider configuration, secret references, registry, invocation policy, OpenAI-compatible adapter, CLI/Desktop boundaries, privacy, and tests.

### Task 3: Verify Documentation

**Files:**
- Read: `docs/specs/M13-00-real-model-providers-config-technical-design.spec.md`

- [x] **Step 1: Check for placeholders**

Run:

```bash
rg -n "TBD|TODO|fill in|implement later" docs/specs/M13-00-real-model-providers-config-technical-design.spec.md
```

Expected: no matches.
