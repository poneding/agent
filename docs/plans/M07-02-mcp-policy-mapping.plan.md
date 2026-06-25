# M07-02 MCP Policy Mapping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert discovered MCP tools into local tool definitions and permission requests.

**Architecture:** `agent-mcp` owns mapping from MCP descriptors to local contracts. It may depend on `agent-tool`, `agent-policy`, and `agent-types`; those crates must not depend on `agent-mcp`.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `serde_json`, existing `agent-tool`, `agent-policy`, `agent-types`, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Confirm Inputs

**Files:**
- Read: `docs/specs/M07-02-mcp-policy-mapping.spec.md`
- Read: `crates/agent-mcp/src/tool.rs`
- Read: `crates/agent-tool/src/definition.rs`
- Read: `crates/agent-policy/src/request.rs`
- Read: `crates/agent-policy/src/capability.rs`

- [x] **Step 1: Review the spec**

Run:

```bash
sed -n '1,260p' docs/specs/M07-02-mcp-policy-mapping.spec.md
```

Expected: the spec defines mapping contracts, security defaults, and focused tests.

### Task 2: Write MCP Policy RED Tests

**Files:**
- Create: `crates/agent-mcp/tests/mcp_policy_mapping.rs`

- [x] **Step 1: Add failing tests**

Create tests with these names:

```rust
#[test]
fn mcp_policy_mapping_normalizes_local_tool_name() {
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let name = agent_mcp::local_tool_name_for_mcp_tool(&server_id, "Read File").unwrap();
    assert_eq!(name.as_str(), "mcp.filesystem.read_file");
}

#[test]
fn mcp_policy_mapping_converts_descriptor_to_tool_definition() {
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let descriptor = agent_mcp::McpToolDescriptor::new(
        server_id,
        "read_file",
        Some("Read File"),
        "Read a file",
        serde_json::json!({"type": "object"}),
    )
    .unwrap();

    let mapped = agent_mcp::mcp_descriptor_to_tool_definition(&descriptor).unwrap();

    assert_eq!(mapped.local_name.as_str(), "mcp.filesystem.read_file");
    assert_eq!(mapped.definition.name.as_str(), "mcp.filesystem.read_file");
    assert_eq!(mapped.definition.description, "Read a file");
    assert_eq!(mapped.definition.input_schema, serde_json::json!({"type": "object"}));
}

#[test]
fn mcp_policy_mapping_builds_default_permission_request() {
    let actor = agent_types::ActorRef::system();
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let descriptor = agent_mcp::McpToolDescriptor::new(
        server_id,
        "read_file",
        None::<String>,
        "Read a file",
        serde_json::json!({"type": "object"}),
    )
    .unwrap();

    let mapping = agent_mcp::default_policy_mapping_for_mcp_tool(&descriptor).unwrap();
    let request = agent_mcp::permission_request_for_mcp_mapping(&mapping, actor.clone());

    assert_eq!(request.actor, actor);
    assert_eq!(request.action, agent_policy::CapabilityAction::Execute);
    assert_eq!(request.scope.kind, agent_policy::ResourceKind::Tool);
    assert_eq!(request.scope.pattern, "mcp:filesystem:read_file");
}

#[test]
fn mcp_policy_mapping_rejects_empty_remote_name() {
    let server_id = agent_mcp::McpServerId::new("filesystem").unwrap();
    let result = agent_mcp::local_tool_name_for_mcp_tool(&server_id, " ");
    assert!(result.is_err());
}
```

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p agent-mcp mcp_policy_mapping
```

Expected: fails because mapping APIs do not exist.

### Task 3: Add Mapping Dependencies

**Files:**
- Modify: `crates/agent-mcp/Cargo.toml`

- [x] **Step 1: Add local dependencies**

Add:

```toml
agent-policy = { path = "../agent-policy" }
agent-tool = { path = "../agent-tool" }
agent-types = { path = "../agent-types" }
```

- [x] **Step 2: Confirm metadata**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: metadata succeeds and there is no dependency cycle.

### Task 4: Implement Tool and Policy Mapping

**Files:**
- Create: `crates/agent-mcp/src/tool_mapping.rs`
- Create: `crates/agent-mcp/src/policy.rs`
- Modify: `crates/agent-mcp/src/lib.rs`

- [x] **Step 1: Add tool mapping**

Implement `McpLocalToolName`, `McpToolDefinition`, `local_tool_name_for_mcp_tool`, and `mcp_descriptor_to_tool_definition`.

- [x] **Step 2: Add policy mapping**

Implement `McpPermissionProfile`, `McpPolicyMapping`, `default_policy_mapping_for_mcp_tool`, and `permission_request_for_mcp_mapping`.

- [x] **Step 3: Export contracts**

Update `lib.rs` to export the new modules and public contracts.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p agent-mcp mcp_policy_mapping
```

Expected: all MCP policy mapping tests pass.

### Task 5: Update Progress and Verify

**Files:**
- Modify: `docs/plans/M07-02-mcp-policy-mapping.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Advance active focus**

Set active focus to `M07-03-skill-discovery`.

- [x] **Step 2: Run standard verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 3: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m07 mcp policy mapping"
```

Expected: commit succeeds with source, tests, and documentation together.
