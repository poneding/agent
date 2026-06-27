import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  deriveBridgeConnectionState,
  drainUiFrames,
  normalizeBridgeError,
  sendUiCommand,
  startSession,
  stopSession,
} from '../bridge';
import type { UiCommandEnvelope } from '../types';

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

function commandEnvelope(prompt: string): UiCommandEnvelope {
  return {
    schema_version: 1,
    command_id: '00000000-0000-0000-0000-000000000100',
    client: {
      kind: 'desktop',
      version: '0.1.0',
    },
    command: {
      type: 'start_turn',
      data: {
        workspace_id: '00000000-0000-0000-0000-000000000001',
        conversation_id: '00000000-0000-0000-0000-000000000002',
        prompt,
      },
    },
  };
}

describe('desktop bridge client', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('serializes session commands through Tauri invoke', async () => {
    invokeMock.mockResolvedValueOnce({
      session_id: 'session-1',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'connected',
      pending_frame_count: 1,
    });
    invokeMock.mockResolvedValueOnce({
      session_id: 'session-1',
      workspace_id: 'workspace-1',
      conversation_id: 'conversation-1',
      state: 'disconnected',
      pending_frame_count: 0,
    });

    await expect(
      startSession({ workspace_path: '/tmp/agent', client_version: '0.1.0' }),
    ).resolves.toMatchObject({ state: 'connected' });
    await expect(stopSession()).resolves.toMatchObject({ state: 'disconnected' });

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'start_session', {
      request: { workspace_path: '/tmp/agent', client_version: '0.1.0' },
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'stop_session');
  });

  it('sends UI commands and drains UI-safe frames', async () => {
    invokeMock.mockResolvedValueOnce({
      command_id: '00000000-0000-0000-0000-000000000100',
      accepted: true,
      state: 'connected',
      pending_frame_count: 1,
    });
    invokeMock.mockResolvedValueOnce([
      {
        type: 'error',
        data: {
          code: 'desktop.bridge.test',
          message: '[redacted]',
          recoverable: true,
        },
      },
    ]);

    await expect(sendUiCommand(commandEnvelope('hello'))).resolves.toMatchObject({
      accepted: true,
    });
    await expect(drainUiFrames(8)).resolves.toHaveLength(1);

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'send_ui_command', {
      envelope: commandEnvelope('hello'),
    });
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'drain_ui_frames', {
      maxCount: 8,
    });
  });

  it('normalizes bridge errors and reconnect state', () => {
    expect(
      normalizeBridgeError({
        code: 'desktop.bridge.command_invalid',
        message: 'prompt cannot be empty',
      }),
    ).toEqual({
      code: 'desktop.bridge.command_invalid',
      message: 'prompt cannot be empty',
      recoverable: true,
    });

    expect(deriveBridgeConnectionState(null)).toBe('disconnected');
    expect(
      deriveBridgeConnectionState({
        session_id: 'session-1',
        workspace_id: 'workspace-1',
        conversation_id: 'conversation-1',
        state: 'connected',
        pending_frame_count: 0,
      }),
    ).toBe('connected');
  });
});
