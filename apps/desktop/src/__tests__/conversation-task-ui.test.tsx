import '@testing-library/jest-dom/vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Composer } from '../components/Composer';
import { ConversationList } from '../components/ConversationList';
import { TaskDetail } from '../components/TaskDetail';
import { TaskList } from '../components/TaskList';
import { TranscriptView } from '../components/TranscriptView';
import {
  createConversationTaskState,
  createStartTurnEnvelope,
  reduceConversationTaskState,
} from '../state/conversation';
import type { DesktopWorkspaceSnapshot, UiJsonFrame } from '../types';

const snapshot: DesktopWorkspaceSnapshot = {
  workspace_id: 'workspace-1',
  active_conversation_id: 'conversation-1',
  conversations: [
    {
      conversation_id: 'conversation-1',
      title: 'Main thread',
      last_activity_at: '2026-06-27T00:00:00Z',
      unread_count: 0,
    },
  ],
  tasks: [
    {
      task_id: 'task-1',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'running',
      summary: 'Implement desktop UI',
      updated_at: '2026-06-27T00:00:00Z',
    },
  ],
};

function eventFrame(payload: unknown): UiJsonFrame {
  return {
    type: 'event',
    data: {
      schema_version: 1,
      event_id: 'event-1',
      seq: null,
      occurred_at: '2026-06-27T00:00:00Z',
      conversation_id: 'conversation-1',
      turn_id: null,
      task_id: null,
      correlation_id: 'correlation-1',
      payload,
    },
  };
}

describe('conversation task UI state', () => {
  afterEach(() => {
    cleanup();
  });

  it('creates empty state with enabled composer defaults', () => {
    const state = createConversationTaskState({
      workspace_id: 'workspace-1',
      active_conversation_id: null,
      conversations: [],
      tasks: [],
    });

    expect(state.conversations).toHaveLength(0);
    expect(state.tasks).toHaveLength(0);
    expect(state.transcript).toHaveLength(0);
    expect(state.composerText).toBe('');
  });

  it('reduces transcript and task notification frames', () => {
    let state = createConversationTaskState(snapshot);

    state = reduceConversationTaskState(
      state,
      eventFrame({ type: 'user_message', data: { message: 'hello' } }),
    );
    state = reduceConversationTaskState(
      state,
      eventFrame({ type: 'model_delta', data: { delta: 'hi' } }),
    );
    state = reduceConversationTaskState(
      state,
      eventFrame({
        type: 'task_notification',
        data: { task_id: 'task-2', kind: 'queued', summary: 'Queued background work' },
      }),
    );

    expect(state.transcript.map((entry) => entry.content)).toEqual(['hello', 'hi']);
    expect(state.tasks.map((task) => task.task_id)).toContain('task-2');
  });

  it('renders conversation list, transcript, composer, task list, and detail', () => {
    const state = reduceConversationTaskState(
      createConversationTaskState(snapshot),
      eventFrame({ type: 'final_response', data: { message: 'done' } }),
    );
    const submit = vi.fn();

    render(
      <>
        <ConversationList
          conversations={state.conversations}
          activeConversationId={state.activeConversationId}
          onSelect={() => undefined}
        />
        <TranscriptView entries={state.transcript} />
        <Composer disabled={false} onSubmitPrompt={submit} />
        <TaskList tasks={state.tasks} activeTaskId={state.activeTaskId} onSelect={() => undefined} />
        <TaskDetail task={state.tasks[0]} />
      </>,
    );

    expect(screen.getByText('Main thread')).toBeInTheDocument();
    expect(screen.getByText('done')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Tasks')).getByText('Implement desktop UI')).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText('Message composer'), {
      target: { value: 'Ship it' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));

    expect(submit).toHaveBeenCalledWith('Ship it');
  });

  it('builds typed start turn commands and disables disconnected submit', () => {
    const envelope = createStartTurnEnvelope({
      workspaceId: 'workspace-1',
      conversationId: 'conversation-1',
      prompt: 'hello',
      clientVersion: '0.1.0',
    });

    expect(envelope.command).toEqual({
      type: 'start_turn',
      data: {
        workspace_id: 'workspace-1',
        conversation_id: 'conversation-1',
        prompt: 'hello',
      },
    });

    render(<Composer disabled onSubmitPrompt={vi.fn()} />);
    expect(screen.getByRole('button', { name: 'Send' })).toBeDisabled();
  });
});
