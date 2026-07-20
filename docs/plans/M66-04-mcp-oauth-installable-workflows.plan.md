# M66-04 OAuth Completion for Installable MCP Workflows Plan

> **For agentic workers:** This slice is a decision gate. Implement OAuth code only if a concrete installable MCP workflow requires it.

**Goal:** Confirm whether M66 needs new OAuth implementation. Current conclusion: no new OAuth code is required for M66-04.

**Architecture:** Existing OAuth contracts live in `ruse-mcp`. CLI/Desktop/support surfaces should consume redacted status only when later workflows need it.

---

### Task 1: Inspect Existing OAuth Foundation

**Files:**
- Read: `crates/ruse-mcp/src/client_http.rs`
- Read: `crates/ruse-mcp/tests/mcp_client_http_oauth_*.rs`
- Read: `crates/ruse-mcp/tests/mcp_client_http_bearer_authorization.rs`
- Read: `docs/roadmap/PRODUCT_ROADMAP.md`
- Read: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Confirm implemented foundation**

Existing runtime coverage includes OAuth metadata, PKCE/state, authorization request planning, callback validation, token exchange, token lease conversion, token source, refresh planning, bearer authorization, and redacted diagnostics.

### Task 2: Confirm Installable Workflow Need

**Files:**
- Read: `docs/specs/M66-01-extension-health-baseline-smoke.spec.md`
- Read: `docs/specs/M66-02-plugin-skill-management-hardening.spec.md`
- Read: `docs/specs/M66-03-mcp-management-compatibility-health.spec.md`

- [x] **Step 1: Compare M66 workflows against OAuth needs**

M66-01 through M66-03 need extension health, management hardening, MCP status health, compatibility diagnostics, and redacted CLI/JSON evidence. They do not require a browser loop, callback server, token storage, or refresh execution.

- [x] **Step 2: Decide deferral**

Document deferral rather than adding unused OAuth code.

### Task 3: Document Decision

**Files:**
- Create: `docs/specs/M66-04-mcp-oauth-installable-workflows.spec.md`
- Create: `docs/plans/M66-04-mcp-oauth-installable-workflows.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `docs/roadmap/PRODUCT_ROADMAP.md`
- Modify: `docs/plans/M66-extensions-mcp-hardening.plan.md`

- [x] **Step 1: Mark M66-04 complete as deferral**

Update roadmap and milestone status to make the OAuth decision explicit.

### Task 4: Verification

**Commands:**

```bash
CARGO_TARGET_DIR=target/m66-dev cargo test -p ruse-mcp oauth
cargo fmt --all -- --check
git diff --check
```

- [x] **Step 1: Run OAuth foundation tests**

Run existing OAuth tests to verify the documented foundation remains intact.

- [x] **Step 2: Run formatting and whitespace checks**

Run fmt and diff whitespace checks.

### Task 5: Commit

- [x] **Step 1: Commit M66-04**

Suggested message:

```text
docs: document m66 oauth workflow deferral
```
