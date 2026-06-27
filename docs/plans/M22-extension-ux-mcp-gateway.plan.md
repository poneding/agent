# M22 Extension UX and MCP Gateway Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make plugins, skills, and MCP servers installable, manageable, permissioned, and executable through a shared gateway.

**Architecture:** `agent-plugin`, `agent-skill`, and `agent-mcp` own extension contracts. `agent-core` coordinates registration and policy. UI clients manage extensions through `agent-ui-bridge`.

**Tech Stack:** Rust 2024 edition, process plugin fixtures, MCP stdio/HTTP transports, serde schemas, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Establish M22 Technical Design

**Files:**
- Create: `docs/specs/M22-00-extension-ux-mcp-gateway-technical-design.spec.md`
- Create: `docs/plans/M22-00-extension-ux-mcp-gateway-technical-design.plan.md`

- [x] **Step 1: Write the M22 technical design baseline**

Define extension registry, plugin install flow, skill root management, MCP gateway, capability review, health reports, and failure isolation.

- [x] **Step 2: Write the M22 design execution plan**

Document slice order and fixture requirements.

### Task 2: Decompose M22 Into Slice Specs and Plans

**Files:**
- Create: `docs/specs/M22-01-extension-registry.spec.md`
- Create: `docs/plans/M22-01-extension-registry.plan.md`
- Create: `docs/specs/M22-02-plugin-install-commands.spec.md`
- Create: `docs/plans/M22-02-plugin-install-commands.plan.md`
- Create: `docs/specs/M22-03-skill-root-management.spec.md`
- Create: `docs/plans/M22-03-skill-root-management.plan.md`
- Create: `docs/specs/M22-04-mcp-gateway-tools.spec.md`
- Create: `docs/plans/M22-04-mcp-gateway-tools.plan.md`
- Create: `docs/specs/M22-05-extension-health-ui.spec.md`
- Create: `docs/plans/M22-05-extension-health-ui.plan.md`

- [x] **Step 1: Create slice specs**

Each slice defines extension state, capability rules, traceability, and tests.

- [x] **Step 2: Create slice plans**

Each plan includes fixture plugins or MCP servers and denial tests.

### Task 3: Implement M22 Slices in Order

**Files:**
- Execute: `docs/plans/M22-01-extension-registry.plan.md`
- Execute: `docs/plans/M22-02-plugin-install-commands.plan.md`
- Execute: `docs/plans/M22-03-skill-root-management.plan.md`
- Execute: `docs/plans/M22-04-mcp-gateway-tools.plan.md`
- Execute: `docs/plans/M22-05-extension-health-ui.plan.md`

- [x] **Step 1: Implement extension registry**

Track installed plugins, skill roots, MCP servers, and enablement state.

- [x] **Step 2: Implement plugin install commands**

Add install, list, enable, disable, remove, and inspect flows.

- [ ] **Step 3: Implement skill root management**

Add skill discovery diagnostics and source tracking.

- [ ] **Step 4: Implement MCP gateway tools**

Expose MCP tools as local tools routed through policy and audit.

- [ ] **Step 5: Implement extension health UI data**

Expose health reports to CLI, TUI, and Desktop bridge clients.

### Task 4: Complete M22

**Files:**
- Modify: `README.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/plans/M22-extension-ux-mcp-gateway.plan.md`

- [ ] **Step 1: Run final workspace gates**

Run:

```bash
cargo xtask check
```

Expected: all checks pass.

- [ ] **Step 2: Mark M22 complete**

Update milestone status and advance active focus to M23.

- [ ] **Step 3: Commit M22 completion**

Run:

```bash
git add crates docs README.md Cargo.toml Cargo.lock
git commit -m "feat: complete m22 extension ux"
```

Expected: commit succeeds.
