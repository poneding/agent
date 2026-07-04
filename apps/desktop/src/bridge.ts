import { invoke } from '@tauri-apps/api/core';

import type {
  DesktopBridgeConnectionState,
  DesktopBridgeError,
  DesktopCommandAck,
  DesktopSessionSnapshot,
  DesktopStartSessionRequest,
  UiCommandEnvelope,
  UiJsonFrame,
} from './types';

export async function startSession(
  request: DesktopStartSessionRequest,
): Promise<DesktopSessionSnapshot> {
  return invoke<DesktopSessionSnapshot>('start_session', { request });
}

export async function stopSession(): Promise<DesktopSessionSnapshot> {
  return invoke<DesktopSessionSnapshot>('stop_session');
}

export async function sendUiCommand(
  envelope: UiCommandEnvelope,
): Promise<DesktopCommandAck> {
  return invoke<DesktopCommandAck>('send_ui_command', { envelope });
}

export async function drainUiFrames(maxCount: number): Promise<UiJsonFrame[]> {
  return invoke<UiJsonFrame[]>('drain_ui_frames', { maxCount });
}

export function normalizeBridgeError(error: unknown): DesktopBridgeError {
  if (isDesktopCommandError(error)) {
    return {
      code: error.code,
      message: error.message,
      recoverable: true,
    };
  }

  return {
    code: 'desktop.bridge.unknown',
    message: error instanceof Error ? error.message : String(error),
    recoverable: true,
  };
}

export function deriveBridgeConnectionState(
  snapshot: DesktopSessionSnapshot | null,
): DesktopBridgeConnectionState {
  if (snapshot === null) {
    return 'disconnected';
  }
  if (snapshot.state === 'connected' || snapshot.state === 'connecting') {
    return snapshot.state;
  }
  if (snapshot.state === 'blocked') {
    return 'blocked';
  }
  return 'disconnected';
}

function isDesktopCommandError(
  error: unknown,
): error is { code: string; message: string } {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    typeof (error as { code?: unknown }).code === 'string' &&
    typeof (error as { message?: unknown }).message === 'string'
  );
}
