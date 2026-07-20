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
  createSwitchSessionEnvelope,
  recordTurnSubmission,
  reduceBridgeError,
  reduceCommandAck,
  reduceConversationTaskFrames,
  reduceConversationTaskState,
  selectConversation,
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
    {
      conversation_id: 'conversation-2',
      title: 'New session',
      last_activity_at: '2026-06-26T00:00:00Z',
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
  return eventFrameFor('conversation-1', payload);
}

function eventFrameFor(conversationId: string, payload: unknown): UiJsonFrame {
  return {
    type: 'event',
    data: {
      schema_version: 1,
      event_id: `event-${conversationId}`,
      seq: null,
      occurred_at: '2026-06-27T00:00:00Z',
      conversation_id: conversationId,
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
    expect(state.currentSubmission).toBeNull();
    expect(state.diagnostics).toHaveLength(0);
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

  it('keeps transcript history and generated titles scoped by conversation', () => {
    let state = createConversationTaskState(snapshot);
    state = reduceConversationTaskState(
      state,
      eventFrameFor('conversation-1', {
        type: 'user_message',
        data: { message: 'message in the main thread' },
      }),
    );
    state = reduceConversationTaskState(
      state,
      eventFrameFor('conversation-2', {
        type: 'user_message',
        data: { message: 'Investigate session switching. Later details' },
      }),
    );

    expect(state.transcript.map((entry) => entry.content)).toEqual([
      'message in the main thread',
    ]);
    expect(
      state.conversations.find(
        (conversation) => conversation.conversation_id === 'conversation-2',
      )?.title,
    ).toBe('Investigate session switching');

    state = selectConversation(state, 'conversation-2');
    expect(state.transcript.map((entry) => entry.content)).toEqual([
      'Investigate session switching. Later details',
    ]);
    state = selectConversation(state, 'conversation-1');
    expect(state.transcript.map((entry) => entry.content)).toEqual([
      'message in the main thread',
    ]);
  });

  it('builds a typed switch-session command', () => {
    const envelope = createSwitchSessionEnvelope({
      workspaceId: 'workspace-1',
      conversationId: 'conversation-2',
      clientVersion: '0.1.0',
    });

    expect(envelope.command).toEqual({
      type: 'switch_session',
      data: {
        workspace_id: 'workspace-1',
        conversation_id: 'conversation-2',
      },
    });
  });

  it('reduces plan todo snapshots into matching tasks', () => {
    let state = createConversationTaskState(snapshot);

    state = reduceConversationTaskState(state, {
      type: 'plan_todo_snapshot',
      data: {
        snapshot: {
          conversation_id: 'conversation-1',
          task_id: 'task-1',
          revision: 4,
          items: [
            {
              item_id: 'todo-1',
              content: 'Apply UI patch',
              status: 'in_progress',
              priority: 'high',
              task_id: 'task-1',
              evidence: [{ kind: 'artifact', reference: 'artifact-1' }],
            },
          ],
          updated_at: '2026-06-27T00:00:02Z',
        },
        progress: {
          total: 3,
          pending: 1,
          in_progress: 1,
          completed: 1,
          cancelled: 0,
        },
        errors: [],
      },
    });

    expect(state.tasks[0].plan_progress?.total).toBe(3);
    expect(state.tasks[0].active_todo?.content).toBe('Apply UI patch');
    expect(state.tasks[0].active_todo?.evidence_count).toBe(1);
  });

  it('tracks submitted prompts, command acknowledgements, and done frames', () => {
    const envelope = createStartTurnEnvelope({
      workspaceId: 'workspace-1',
      conversationId: 'conversation-1',
      prompt: 'inspect token sk-test-secret and report',
      clientVersion: '0.1.0',
    });
    let state = recordTurnSubmission(createConversationTaskState(snapshot), envelope);

    expect(state.currentSubmission?.status).toBe('submitting');
    expect(state.currentSubmission?.prompt_preview).toContain('[redacted]');
    expect(state.currentSubmission?.prompt_preview).not.toContain('sk-test');

    state = reduceCommandAck(state, {
      command_id: envelope.command_id,
      accepted: true,
      state: 'connected',
      pending_frame_count: 2,
    });
    expect(state.currentSubmission?.status).toBe('accepted');
    expect(state.pendingFrameCount).toBe(2);

    state = reduceConversationTaskFrames(state, [
      eventFrame({ type: 'final_response', data: { message: 'done' } }),
      { type: 'done', data: { event_count: 1 } },
    ]);

    expect(state.currentSubmission?.status).toBe('completed');
    expect(state.pendingFrameCount).toBe(0);
    expect(state.transcript.map((entry) => entry.content)).toEqual([
      'done',
      '1 frame events processed',
    ]);
  });

  it('turns bridge errors into recoverable diagnostics', () => {
    const envelope = createStartTurnEnvelope({
      workspaceId: 'workspace-1',
      conversationId: 'conversation-1',
      prompt: 'hello',
      clientVersion: '0.1.0',
    });
    let state = recordTurnSubmission(createConversationTaskState(snapshot), envelope);

    state = reduceBridgeError(state, {
      code: 'desktop.bridge.disconnected',
      message: 'token sk-test-secret is hidden',
      recoverable: true,
    });

    expect(state.currentSubmission?.status).toBe('rejected');
    expect(state.currentSubmission?.error?.message).toContain('[redacted]');
    expect(state.currentSubmission?.error?.message).not.toContain('sk-test');
    expect(state.diagnostics[0]).toMatchObject({
      severity: 'warning',
      code: 'desktop.bridge.disconnected',
    });
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
        <Composer
          disabled={false}
          onSubmitPrompt={submit}
          submission={{
            command_id: 'command-1',
            conversation_id: 'conversation-1',
            prompt_preview: 'Ship it',
            status: 'accepted',
            ack: {
              command_id: 'command-1',
              accepted: true,
              pending_frame_count: 1,
            },
            error: null,
            truncated: false,
          }}
        />
        <TaskList tasks={state.tasks} activeTaskId={state.activeTaskId} onSelect={() => undefined} />
        <TaskDetail task={state.tasks[0]} />
      </>,
    );

    expect(screen.getByText('Main thread')).toBeInTheDocument();
    expect(screen.getByText('done')).toBeInTheDocument();
    expect(screen.getByText('accepted')).toBeInTheDocument();
    expect(screen.getByText('1 pending frames')).toBeInTheDocument();
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
