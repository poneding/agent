# MX02-02 Thread and Composer Experience Spec

## Purpose

Make the Desktop center workspace feel like a real coding-agent thread: searchable thread lifecycle controls, a capable composer, typed transcript event rendering, slash command hints, skill hints, and visible goal/progress state.

## Product Context

MX02-01 creates the shell. MX02-02 fills the left/center workflow around threads and prompt submission while continuing to consume typed bridge frames and Desktop view models.

## Scope

This spec delivers:

- thread lifecycle UI for new, resume, switch, pin, archive, search, and find-in-thread
- composer provider selector, model selector, Local/Worktree segmented control, permission mode indicator, context toggle placeholder, slash command menu, `$` skill hint, send/stop/retry states, and goal/progress row
- transcript event rendering for user, assistant, tool, approval, diff, task, and system event types
- compact tool cards and expandable command output
- redaction behavior that avoids raw secret output in transcript, command cards, diagnostics, snapshots, and tests

Owned by other specs:

- provider/model configuration persistence belongs to MX02-03
- inspector-specific approval/diff/task/artifact panels belong to MX02-04
- recovery-aware resume/replay/fork belongs to M62

## Architecture

Thread state is derived from `agent-desktop` conversation/task snapshots and `agent-ui-bridge` frames. React may track search/filter/pin/archive presentation state until durable thread metadata exists, but runtime status and transcript entries come from typed frames.

## File Layout

```text
crates/
  agent-desktop/src/conversation.rs
  agent-ui-bridge/src/command.rs
  agent-ui-bridge/src/frame.rs
apps/desktop/src/
  components/Composer.tsx
  components/ConversationList.tsx
  components/TranscriptView.tsx
  state/conversation.ts
  types.ts
  __tests__/conversation-task-ui.test.tsx
  __tests__/app-shell.test.tsx
```

## Public Contracts

Potential bridge/Desktop extensions:

```rust
pub struct DesktopThreadSummary {
    pub conversation_id: ConversationId,
    pub title: String,
    pub workspace_label: String,
    pub last_activity_at: Timestamp,
    pub pinned: bool,
    pub archived: bool,
    pub unread_count: u32,
}

pub struct DesktopComposerState {
    pub provider_id: String,
    pub model_id: String,
    pub workspace_mode: DesktopWorkspaceMode,
    pub permission_mode_label: String,
    pub active_goal: Option<DesktopGoalProgress>,
}
```

## State and Failure Handling

- New-thread, pin, archive, and resume actions remain UI placeholders until typed commands exist; implemented actions must dispatch typed bridge commands.
- Composer send is disabled without an active connected session, active conversation, provider, and model.
- Stop/retry states are visible but do not bypass runtime cancellation/retry contracts.
- Unknown frame kinds render as bounded system events instead of crashing.

## Security and Privacy

- Transcript entries, tool cards, command output, and diagnostics are bounded and redacted.
- Provider/model selectors expose ids and display names, not secrets.
- Worktree mode is a UI selection only until a typed runtime command validates it.

## Testing Strategy

- Vitest coverage for thread sections, search/filtering, composer controls, slash menu, `$` hint, goal/progress row, typed transcript roles, tool cards, expandable command output, and redaction
- Build check for TypeScript and bundling

## Acceptance Criteria

- Users can see and manage thread lists for recent, pinned, and archived work.
- Composer exposes provider/model, mode, permission, context, slash, skill, and send/stop/retry affordances.
- Transcript renders typed event kinds compactly and safely.
- Reducers consume typed bridge frames only.

## Completion Signal

MX02-02 is complete when `npm --prefix apps/desktop test -- conversation-task-ui app-shell` and `npm --prefix apps/desktop run build` pass.
