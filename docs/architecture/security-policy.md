# Security Policy

Security in `agent` is based on least privilege, explicit capability grants, sandboxed execution, and auditable decisions.

## Trust Model

Treat these inputs as untrusted:

- repository contents
- model output
- tool output
- MCP server responses
- plugin manifests and outputs
- browser pages
- memory records from prior sessions
- user-provided files

## Capability Categories

| Category | Examples |
|---|---|
| filesystem | read, write, delete, glob scope |
| command | executable, arguments, environment, cwd |
| network | domain, port, method |
| git | status, diff, commit, push, branch, worktree |
| MCP | server, tool, resource |
| browser | navigate, click, type, screenshot, clipboard |
| desktop | window, notification, file dialog |
| secret | named secret read, scoped injection |
| scheduler | spawn task, set cron, change quota |
| subagent | delegate role, inherit or reduce capability |

## Decision Model

Policy decisions are:

- `allow`
- `deny`
- `ask`
- `defer`

Every decision records actor, resource, action, scope, reason, and lease duration when applicable.

## Sandboxing

Sandboxing is the technical boundary. Approval is the interaction boundary. A user approval must not bypass sandbox limits unless policy explicitly changes the sandbox profile.

Sandbox profiles cover:

- filesystem roots
- environment allowlist
- network access
- process timeout
- stdout/stderr limits
- working directory
- platform capabilities

## Secrets

Secret values are injected only into approved tool execution contexts. They are not written to prompts, events, traces, logs, snapshots, memory, or artifacts.

## Subagents

Subagent capability is the intersection of:

- parent capability
- role capability
- task capability
- workspace policy

Subagents cannot grant themselves broader access.

## Audit Requirements

Audit records are durable and queryable. They include permission requests, decisions, leases, tool execution summaries, sandbox profile, actor, and correlation IDs.
