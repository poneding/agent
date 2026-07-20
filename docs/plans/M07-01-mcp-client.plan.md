# M07-01 MCP Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `ruse-mcp` with deterministic MCP server config, initialization, tool discovery, and health-check contracts.

**Architecture:** `ruse-mcp` owns MCP protocol-facing contracts and fixture-backed client behavior. It stays independent from `ruse-core`, UI crates, storage, model providers, and real transports in this slice.

**Tech Stack:** Rust 2024 edition, Cargo workspace, `serde`, `serde_json`, `thiserror`, `async-trait`, `tokio` test runtime, Cargo fmt, Cargo clippy, Cargo test.

---

### Task 1: Scaffold `ruse-mcp`

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ruse-mcp/Cargo.toml`
- Create: `crates/ruse-mcp/src/lib.rs`
- Create: `crates/ruse-mcp/src/error.rs`

- [x] **Step 1: Add crate scaffold**

Add `crates/ruse-mcp` to workspace members in `Cargo.toml`.

Create `crates/ruse-mcp/Cargo.toml`:

```toml
[package]
name = "ruse-mcp"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt"] }

[lints]
workspace = true
```

Create `src/lib.rs`:

```rust
#![forbid(unsafe_code)]
//! MCP protocol contracts and deterministic client fixtures for agent.

pub mod client;
pub mod config;
pub mod error;
pub mod tool;

pub use client::{
    check_mcp_server_health, FixtureMcpClient, McpClient, McpInitializeResult,
    McpServerCapabilities, McpServerHealth, McpServerStatus,
};
pub use config::{McpEnvValue, McpServerConfig, McpServerId, McpTransportConfig};
pub use error::McpError;
pub use tool::{McpToolDescriptor, McpToolListResult};
```

Create `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum McpError {
    #[error("invalid MCP config: {0}")]
    InvalidConfig(String),
    #[error("invalid MCP tool descriptor: {0}")]
    InvalidToolDescriptor(String),
    #[error("MCP server not found: {0}")]
    ServerNotFound(String),
    #[error("MCP client error: {0}")]
    Client(String),
}
```

- [x] **Step 2: Confirm scaffold compiles**

Run:

```bash
cargo test -p ruse-mcp
```

Expected: the crate is wired into the workspace; the behavior RED check is performed by the focused tests in Task 2.

### Task 2: Write MCP Client RED Tests

**Files:**
- Create: `crates/ruse-mcp/tests/mcp_client.rs`

- [x] **Step 1: Add failing tests**

Create tests with these names:

```rust
#[tokio::test]
async fn mcp_client_validates_server_config() {
    assert!(ruse_mcp::McpServerId::new("filesystem").is_ok());
    assert!(ruse_mcp::McpServerId::new("FileSystem").is_err());
}

#[tokio::test]
async fn mcp_client_serializes_transport_shape() {
    use std::collections::BTreeMap;

    let config = ruse_mcp::McpServerConfig::new(
        ruse_mcp::McpServerId::new("docs").unwrap(),
        "Docs",
        ruse_mcp::McpTransportConfig::Stdio {
            command: "mcp-docs".to_string(),
            args: vec!["--stdio".to_string()],
            env: BTreeMap::from([(
                "API_TOKEN".to_string(),
                ruse_mcp::McpEnvValue::SecretRef("docs.token".to_string()),
            )]),
        },
    )
    .unwrap();

    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(json["transport"]["kind"], "stdio");
    assert_eq!(json["transport"]["env"]["API_TOKEN"]["kind"], "secret_ref");
}

#[tokio::test]
async fn mcp_client_fixture_initializes_and_lists_tools() {
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let config = ruse_mcp::McpServerConfig::new(
        server_id.clone(),
        "Filesystem",
        ruse_mcp::McpTransportConfig::stdio("mcp-filesystem"),
    )
    .unwrap();

    let client = ruse_mcp::FixtureMcpClient::new()
        .with_server(
            server_id.clone(),
            ruse_mcp::McpInitializeResult::new(
                server_id.clone(),
                "2025-11-25",
                "filesystem",
                Some("1.0.0"),
                ruse_mcp::McpServerCapabilities {
                    tools: true,
                    prompts: false,
                    resources: false,
                },
            ),
        )
        .with_tools(
            server_id.clone(),
            vec![ruse_mcp::McpToolDescriptor::new(
                server_id.clone(),
                "read_file",
                Some("Read file"),
                "Read a file from the MCP server",
                serde_json::json!({"type": "object"}),
            )
            .unwrap()],
        );

    let init = client.initialize(&config).await.unwrap();
    let tools = client.list_tools(&server_id).await.unwrap();

    assert_eq!(init.protocol_version, "2025-11-25");
    assert_eq!(tools.tools.len(), 1);
    assert_eq!(tools.tools[0].name, "read_file");
}

#[tokio::test]
async fn mcp_client_health_reports_disabled_healthy_and_unhealthy() {
    let server_id = ruse_mcp::McpServerId::new("filesystem").unwrap();
    let enabled = ruse_mcp::McpServerConfig::new(
        server_id.clone(),
        "Filesystem",
        ruse_mcp::McpTransportConfig::stdio("mcp-filesystem"),
    )
    .unwrap();
    let disabled = enabled.clone().disabled();
    let empty_client = ruse_mcp::FixtureMcpClient::new();

    let disabled_health = ruse_mcp::check_mcp_server_health(&empty_client, &disabled).await;
    assert_eq!(disabled_health.status, ruse_mcp::McpServerStatus::Disabled);

    let unhealthy = ruse_mcp::check_mcp_server_health(&empty_client, &enabled).await;
    assert_eq!(unhealthy.status, ruse_mcp::McpServerStatus::Unhealthy);

    let healthy_client = ruse_mcp::FixtureMcpClient::new()
        .with_server(
            server_id.clone(),
            ruse_mcp::McpInitializeResult::new(
                server_id.clone(),
                "2025-11-25",
                "filesystem",
                None,
                ruse_mcp::McpServerCapabilities {
                    tools: true,
                    prompts: false,
                    resources: false,
                },
            ),
        )
        .with_tools(server_id, Vec::new());

    let healthy = ruse_mcp::check_mcp_server_health(&healthy_client, &enabled).await;
    assert_eq!(healthy.status, ruse_mcp::McpServerStatus::Healthy);
}
```

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p ruse-mcp mcp_client
```

Expected: fails because config, client, and tool contracts do not exist.

### Task 3: Implement Config and Tool Contracts

**Files:**
- Create: `crates/ruse-mcp/src/config.rs`
- Create: `crates/ruse-mcp/src/tool.rs`

- [x] **Step 1: Add config contracts**

Implement:

```rust
use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::McpError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct McpServerId(String);

impl McpServerId {
    pub fn new(value: impl Into<String>) -> Result<Self, McpError> {
        let value = value.into();
        if value.is_empty()
            || !value
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.'))
        {
            return Err(McpError::InvalidConfig(format!("invalid server id `{value}`")));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: McpServerId,
    pub display_name: String,
    pub transport: McpTransportConfig,
    pub enabled: bool,
    pub timeout_ms: u64,
}

impl McpServerConfig {
    pub fn new(
        id: McpServerId,
        display_name: impl Into<String>,
        transport: McpTransportConfig,
    ) -> Result<Self, McpError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() {
            return Err(McpError::InvalidConfig("display name cannot be empty".to_string()));
        }
        transport.validate()?;
        Ok(Self {
            id,
            display_name,
            transport,
            enabled: true,
            timeout_ms: 30_000,
        })
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, McpEnvValue>,
    },
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, McpEnvValue>,
    },
}

impl McpTransportConfig {
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
        }
    }

    fn validate(&self) -> Result<(), McpError> {
        match self {
            Self::Stdio { command, .. } if command.trim().is_empty() => {
                Err(McpError::InvalidConfig("stdio command cannot be empty".to_string()))
            }
            Self::StreamableHttp { url, .. }
                if !(url.starts_with("http://") || url.starts_with("https://")) =>
            {
                Err(McpError::InvalidConfig("streamable http url must start with http:// or https://".to_string()))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum McpEnvValue {
    Literal(String),
    SecretRef(String),
}
```

- [x] **Step 2: Add tool contracts**

Implement `McpToolDescriptor` and `McpToolListResult` with validation matching the spec.

- [x] **Step 3: Run focused tests**

Run:

```bash
cargo test -p ruse-mcp mcp_client_validates_server_config mcp_client_serializes_transport_shape
```

Expected: config and serialization tests pass.

### Task 4: Implement Fixture Client and Health Checks

**Files:**
- Create: `crates/ruse-mcp/src/client.rs`

- [x] **Step 1: Add client contracts**

Implement:

```rust
use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{McpError, McpServerConfig, McpServerId, McpToolListResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    pub tools: bool,
    pub prompts: bool,
    pub resources: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpInitializeResult {
    pub server_id: McpServerId,
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: Option<String>,
    pub capabilities: McpServerCapabilities,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerStatus {
    Disabled,
    NotInitialized,
    Healthy,
    Unhealthy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct McpServerHealth {
    pub server_id: McpServerId,
    pub status: McpServerStatus,
    pub message: String,
}

#[async_trait]
pub trait McpClient: Send + Sync {
    async fn initialize(&self, config: &McpServerConfig) -> Result<McpInitializeResult, McpError>;
    async fn list_tools(&self, server_id: &McpServerId) -> Result<McpToolListResult, McpError>;
}

#[derive(Clone, Debug, Default)]
pub struct FixtureMcpClient {
    servers: BTreeMap<McpServerId, McpInitializeResult>,
    tools: BTreeMap<McpServerId, McpToolListResult>,
}
```

Add `new`, `with_server`, `with_tools`, `McpClient` implementation, and `check_mcp_server_health`.

- [x] **Step 2: Verify GREEN**

Run:

```bash
cargo test -p ruse-mcp mcp_client
```

Expected: all `mcp_client` tests pass.

### Task 5: Update Progress and Verify

**Files:**
- Modify: `docs/plans/M07-01-mcp-client.plan.md`
- Modify: `docs/milestones/README.md`
- Modify: `README.md`

- [x] **Step 1: Mark plan checkboxes complete**

Update this plan as each task finishes.

- [x] **Step 2: Advance active focus**

Set active focus to `M07-02-mcp-policy-mapping`.

- [x] **Step 3: Run standard verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all checks pass.

- [x] **Step 4: Commit**

Run:

```bash
git add Cargo.toml Cargo.lock crates docs README.md AGENTS.md
git commit -m "feat: add m07 mcp client"
```

Expected: commit succeeds with source, tests, and documentation together.
