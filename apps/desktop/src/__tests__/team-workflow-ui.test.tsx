import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { AppShell } from '../App';
import { TeamWorkflowView } from '../components/TeamWorkflowView';
import { createConversationTaskState, reduceConversationTaskState } from '../state/conversation';
import type {
  DesktopBootstrap,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
  UiTeamWorkflowSnapshot,
} from '../types';

const teamWorkflow: UiTeamWorkflowSnapshot = {
  operation: 'merge_readiness_dry_run',
  dry_run: true,
  snapshot: {
    team_id: 'team-1',
    lead_agent_id: 'lead-1',
    workspace_id: 'workspace-1',
    task_id: 'task-1',
    status: 'blocked',
    readiness: 'has_conflict',
    worktrees: [
      {
        lease_id: 'worktree-team',
        base_branch: 'main',
        worktree_branch: 'agent/m63-04',
        worktree_path: '.worktrees/agent-m63-04',
        state: 'active',
      },
    ],
    handoffs: [
      {
        subagent_id: 'subagent-1',
        role: 'implementer',
        worktree_lease_id: 'worktree-team',
        status: 'completed',
        objective: 'Implement UI team surface',
      },
    ],
    blackboard: {
      entry_count: 1,
      latest_entries: [
        {
          kind: 'review',
          summary: 'Reviewer found conflict',
          artifact_count: 1,
          trace_event_count: 2,
          review_vote: 'block',
        },
      ],
      approve_count: 0,
      request_changes_count: 0,
      block_count: 1,
    },
    merge_review: {
      required_review_count: 1,
      approving_review_count: 0,
      blocking_review_count: 1,
      artifact_count: 1,
      approved: false,
    },
    capability_ceiling: {
      grant_count: 1,
      grants: [
        {
          action: 'read',
          scope_kind: 'file',
          scope_pattern: 'crates/agent-team/**',
        },
      ],
      policy_blocked: false,
    },
    pending_approval_count: 0,
    conflicts: [
      {
        path: 'crates/agent-team/src/workflow.rs',
        kind: 'both_modified',
        lead_change: {
          source: 'lead',
          change_kind: 'modified',
          content_hash: 'lead',
        },
        incoming_change: {
          source: 'subagent',
          change_kind: 'modified',
          content_hash: 'incoming',
        },
        next_action: 'inspect_diff',
        diagnostics: [],
      },
    ],
    diagnostics: [
      {
        severity: 'warning',
        code: 'team.ui.fixture',
        message: 'team UI fixture',
      },
    ],
  },
  errors: [],
};

const bootstrap: DesktopBootstrap = {
  app_name: 'agent',
  app_version: '0.1.0',
  runtime_state: 'connected',
  route: 'tasks',
  diagnostics: [],
};

const workspaceSnapshot: DesktopWorkspaceSnapshot = {
  workspace_id: 'workspace-1',
  active_conversation_id: 'conversation-1',
  conversations: [],
  tasks: [
    {
      task_id: 'task-1',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'running',
      summary: 'Implement team UI',
      updated_at: '2026-07-03T00:00:00Z',
    },
  ],
};

const sessionSnapshot: DesktopSessionSnapshot = {
  session_id: 'session-1',
  workspace_id: 'workspace-1',
  conversation_id: 'conversation-1',
  state: 'connected',
  pending_frame_count: 0,
};

describe('team workflow UI', () => {
  afterEach(() => {
    cleanup();
  });

  it('renders team workflow readiness and evidence from a bridge snapshot', () => {
    render(<TeamWorkflowView snapshot={teamWorkflow} />);

    const panel = screen.getByLabelText('Team workflow');
    expect(within(panel).getByText('has_conflict')).toBeInTheDocument();
    expect(within(panel).getByText('merge_readiness_dry_run')).toBeInTheDocument();
    expect(within(panel).getByText('1 grants, policy clear')).toBeInTheDocument();
    expect(within(panel).getByText('0/1 approvals, 1 blocks, 1 artifacts')).toBeInTheDocument();
    expect(within(panel).getByText('Implement UI team surface')).toBeInTheDocument();
    expect(within(panel).getByText('crates/agent-team/src/workflow.rs')).toBeInTheDocument();
    expect(within(panel).getByText('team.ui.fixture')).toBeInTheDocument();
  });

  it('shows team workflow panel in the tasks inspector tab', () => {
    const conversationState = reduceConversationTaskState(createConversationTaskState(workspaceSnapshot), {
      type: 'team_workflow_snapshot',
      data: teamWorkflow,
    });

    render(
      <AppShell
        bootstrap={bootstrap}
        bootstrapError={null}
        workspaceSnapshot={workspaceSnapshot}
        conversationState={conversationState}
        sessionSnapshot={sessionSnapshot}
      />,
    );

    const inspector = screen.getByRole('complementary', { name: 'Inspector' });
    fireEvent.click(within(inspector).getByRole('tab', { name: 'Tasks' }));
    const tasksPanel = within(inspector).getByRole('tabpanel', { name: 'Tasks' });
    const taskList = within(tasksPanel).getByRole('list', { name: 'Tasks' });
    expect(within(tasksPanel).getByLabelText('Team workflow')).toBeInTheDocument();
    expect(within(taskList).getByText('Implement team UI')).toBeInTheDocument();
    expect(
      within(within(tasksPanel).getByLabelText('Team workflow')).getByText('has_conflict'),
    ).toBeInTheDocument();
  });
});
