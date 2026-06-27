import { invoke } from '@tauri-apps/api/core';

import type {
  DesktopBootstrap,
  DesktopCommandAck,
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopResolveApprovalRequest,
  DesktopSettings,
  DesktopWorkspaceSnapshot,
  TraceTimeline,
} from './types';

export async function desktopBootstrap(): Promise<DesktopBootstrap> {
  return invoke<DesktopBootstrap>('desktop_bootstrap');
}

export async function loadWorkspaceSnapshot(): Promise<DesktopWorkspaceSnapshot> {
  return invoke<DesktopWorkspaceSnapshot>('load_workspace_snapshot');
}

export async function loadTrace(conversationId: string): Promise<TraceTimeline> {
  return invoke<TraceTimeline>('load_trace', {
    conversationId,
  });
}

export async function loadConfigSnapshot(): Promise<DesktopConfigSnapshot> {
  return invoke<DesktopConfigSnapshot>('load_config_snapshot');
}

export async function validateSettings(
  settings: DesktopSettings,
): Promise<DesktopBootstrapDiagnostic[]> {
  return invoke<DesktopBootstrapDiagnostic[]>('validate_settings', {
    settings,
  });
}

export async function resolveApproval(
  request: DesktopResolveApprovalRequest,
  clientVersion: string,
): Promise<DesktopCommandAck> {
  return invoke<DesktopCommandAck>('resolve_approval', {
    request,
    clientVersion,
  });
}
