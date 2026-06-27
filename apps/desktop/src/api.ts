import { invoke } from '@tauri-apps/api/core';

import type {
  DesktopBootstrap,
  DesktopCommandAck,
  DesktopResolveApprovalRequest,
  DesktopWorkspaceSnapshot,
} from './types';

export async function desktopBootstrap(): Promise<DesktopBootstrap> {
  return invoke<DesktopBootstrap>('desktop_bootstrap');
}

export async function loadWorkspaceSnapshot(): Promise<DesktopWorkspaceSnapshot> {
  return invoke<DesktopWorkspaceSnapshot>('load_workspace_snapshot');
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
