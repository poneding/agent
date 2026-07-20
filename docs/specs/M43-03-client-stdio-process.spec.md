# M43-03 Client Stdio Process Spec

## Purpose

Add a spawned process implementation of the MCP client stdio transport.

## Product Context

M43-02 proves the session runtime over fixtures. M43-03 connects that runtime to external MCP server processes while keeping process IO isolated and safe.

## Scope

This spec delivers:

- process spawn from `McpTransportConfig::Stdio`
- stdin write and stdout line read transport implementation
- timeout, EOF, process exit, and cancellation handling
- redacted diagnostics for stderr and spawn failures

Owned by other specs:

- typed initialize/tools/list methods
- CLI user-facing commands

## Architecture

`ruse-mcp` owns process transport. It uses Tokio process and IO features behind a public transport type. Secret resolution and user prompts remain outside `ruse-mcp`.

## Public Contracts

Expected contracts:

```rust
pub struct McpClientStdioProcessTransport;

impl McpClientStdioProcessTransport {
    pub async fn spawn(config: &McpServerConfig) -> Result<Self, McpError>;
}
```

## State and Failure Handling

- Disabled configs fail before spawn.
- Non-stdio configs fail before spawn.
- Child stdin/stdout absence is a startup error.
- EOF before response is a transport error.
- Drop or close attempts to terminate the child process.

## Security and Privacy

- Public errors must not expose raw command env, secret refs, stderr, prompt text, or tool arguments.
- Command and args are config data and may be summarized only after bounding and redaction.

## Testing Strategy

- subprocess fixture serves initialize/list frames
- disabled config rejects before spawn
- process exit before response returns a redacted error
- stderr secret-like content does not leak

## Acceptance Criteria

- `CARGO_TARGET_DIR=target/codex-m43 cargo test -p ruse-mcp mcp_client_stdio_process`

## Completion Signal

M43-03 is complete when a local fixture process can exchange MCP JSON-RPC frames through the client stdio transport.
