import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { AppShell } from '../App';
import { MemoryView } from '../components/MemoryView';
import { createConversationTaskState, reduceConversationTaskState } from '../state/conversation';
import type {
  DesktopBootstrap,
  DesktopSessionSnapshot,
  DesktopWorkspaceSnapshot,
  UiMemorySnapshot,
} from '../types';

const memorySnapshot: UiMemorySnapshot = {
  scope_label: 'runtime memory',
  enabled: true,
  layer_count: 1,
  record_count: 2,
  proposal_count: 1,
  selected_count: 1,
  rejected_count: 1,
  tombstone_count: 1,
  records_truncated: false,
  proposals_truncated: false,
  retrieval_candidates_truncated: false,
  influence_entries_truncated: false,
  diagnostics_truncated: false,
  records: [
    {
      id: 'memory-public',
      scope: 'project:workspace-1',
      sensitivity: 'public',
      confidence: 91,
      source_description: 'memory bridge fixture',
      text_preview: 'prefer deterministic memory UI surfaces',
      preview_redacted: false,
      preview_truncated: false,
    },
    {
      id: 'memory-sensitive',
      scope: 'project:workspace-1',
      sensitivity: 'sensitive',
      confidence: 88,
      source_description: 'memory bridge fixture',
      text_preview: '[redacted]',
      preview_redacted: true,
      preview_truncated: false,
    },
  ],
  proposals: [
    {
      proposal_id: 'proposal-review',
      proposed_record_id: 'record-review',
      scope: 'conversation:conversation-1',
      summary: 'remember review preference for [redacted] values',
      confidence: 88,
      sensitivity: 'sensitive',
      status: 'pending',
      content_redacted: true,
      requires_explicit_approval: true,
      auto_approval_eligible: false,
      auto_approval_reason: 'proposal requires explicit approval',
      source_event_count: 1,
    },
  ],
  retrieval: {
    query_hash: 'fnv1a64:memory',
    selected_count: 1,
    rejected_count: 1,
    candidates: [
      {
        memory_id: 'memory-public',
        scope: 'project:workspace-1',
        sensitivity: 'public',
        selected: true,
        rank: 0,
        token_estimate: 12,
        lexical_score: '0.800',
        semantic_score: '0.400',
        combined_score: '0.900',
        matched_terms: ['memory'],
        reasons: ['score'],
        preview: 'prefer deterministic memory UI surfaces',
        preview_redacted: false,
        preview_truncated: false,
      },
      {
        memory_id: 'memory-sensitive',
        scope: 'project:workspace-1',
        sensitivity: 'sensitive',
        selected: false,
        rank: 1,
        token_estimate: 16,
        lexical_score: '0.400',
        combined_score: '0.500',
        matched_terms: ['[redacted]'],
        reasons: ['budget'],
        preview: '[redacted]',
        preview_redacted: true,
        preview_truncated: false,
      },
    ],
    candidates_truncated: false,
  },
  influence: {
    query_hash: 'fnv1a64:memory',
    selected_count: 1,
    rejected_count: 1,
    entries: [
      {
        memory_id: 'memory-public',
        scope: 'project:workspace-1',
        sensitivity: 'public',
        selected: true,
        rank: 0,
        lexical_score: '0.800',
        semantic_score: '0.400',
        combined_score: '0.900',
        matched_terms: ['memory'],
        reasons: ['score'],
        preview: 'prefer deterministic memory UI surfaces',
        preview_redacted: false,
        preview_truncated: false,
      },
    ],
    entries_truncated: false,
  },
  retention: {
    memory_enabled: true,
    export_includes_sensitive: false,
    tombstones_retained: true,
    sensitive_memory_requires_review: true,
  },
  diagnostics: [
    {
      severity: 'warning',
      code: 'memory.degraded',
      message: 'memory index unavailable; showing stored summaries only',
    },
  ],
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
      summary: 'Review memory surface',
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

describe('memory UI', () => {
  afterEach(() => {
    cleanup();
  });

  it('renders memory state from a bridge snapshot without raw sensitive content', () => {
    render(<MemoryView snapshot={memorySnapshot} />);

    const panel = screen.getByLabelText('Memory');
    expect(within(panel).getByText('enabled')).toBeInTheDocument();
    expect(within(panel).getAllByText('memory-public').length).toBeGreaterThan(0);
    expect(within(panel).getAllByText('memory-sensitive').length).toBeGreaterThan(0);
    expect(within(panel).getAllByText('[redacted]').length).toBeGreaterThan(0);
    expect(within(panel).getByText('proposal-review')).toBeInTheDocument();
    expect(within(panel).getByText('pending')).toBeInTheDocument();
    expect(within(panel).getByLabelText('Memory retrieval')).toHaveTextContent(
      '1 selected, 1 rejected',
    );
    expect(within(panel).getByLabelText('Memory influence')).toHaveTextContent(
      '1 selected, 1 rejected',
    );
    expect(within(panel).getByText('memory.degraded')).toBeInTheDocument();
    expect(panel).not.toHaveTextContent('sk-secret');
    expect(panel).not.toHaveTextContent('password');
    expect(panel).not.toHaveTextContent('remember raw');
  });

  it('stores memory snapshot frames in conversation state', () => {
    const state = reduceConversationTaskState(createConversationTaskState(workspaceSnapshot), {
      type: 'memory_snapshot',
      data: memorySnapshot,
    });

    expect(state.memorySnapshot?.record_count).toBe(2);
    expect(state.memorySnapshot?.proposal_count).toBe(1);
    expect(state.memorySnapshot?.diagnostics[0]?.code).toBe('memory.degraded');
  });

  it('shows memory panel in the tasks inspector tab', () => {
    const conversationState = reduceConversationTaskState(
      createConversationTaskState(workspaceSnapshot),
      {
        type: 'memory_snapshot',
        data: memorySnapshot,
      },
    );

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
    expect(within(tasksPanel).getByLabelText('Memory')).toBeInTheDocument();
    expect(
      within(within(tasksPanel).getByLabelText('Memory')).getByText('proposal-review'),
    ).toBeInTheDocument();
    expect(within(tasksPanel).getAllByText('Review memory surface').length).toBeGreaterThan(0);
  });
});
