# M22-00 Extension UX and MCP Gateway Technical Design Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the M22 architecture baseline for extension management and MCP gateway execution.

**Architecture:** This is a documentation-only slice. It defines extension registry ownership, plugin install flows, skill root diagnostics, MCP gateway tool mapping, capability review, health reports, and security boundaries.

**Tech Stack:** Markdown docs, existing plugin/skill/MCP contracts, Cargo workspace verification.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M22-extension-ux-mcp-gateway.spec.md`
- Read: `docs/specs/M07-mcp-skills.spec.md`
- Read: `docs/specs/M11-00-plugin-runtime-technical-design.spec.md`
- Read: `docs/architecture/plugin-system.md`

- [ ] **Step 1: Review milestone scope**

Run:

```bash
sed -n '1,220p' docs/specs/M22-extension-ux-mcp-gateway.spec.md
```

Expected: milestone scope defines extension UX, MCP gateway, health, and capability review.

### Task 2: Write Technical Design

**Files:**
- Create: `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md`

- [ ] **Step 1: Define ownership and contracts**

Document registry state, plugin management, skill roots, MCP gateway, CLI/UI surfaces, security, and testing strategy.

- [ ] **Step 2: Self-review design**

Check that extension manifests do not grant permissions, MCP tools route through local policy, and extension output is treated as untrusted.

### Task 3: Verify

**Files:**
- Read: `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md`

- [ ] **Step 1: Run workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

### Task 4: Commit

**Files:**
- Add: `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md`
- Add: `docs/plans/M22-00-extension-ux-mcp-gateway-technical-design.plan.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`

- [ ] **Step 1: Commit design docs**

Run:

```bash
git add docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md docs/plans/M22-00-extension-ux-mcp-gateway-technical-design.plan.md docs/plans/M22-extension-ux-mcp-gateway.plan.md
git commit -m "docs: add m22 technical design"
```

Expected: commit succeeds if run as a standalone design slice.
