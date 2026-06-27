import { invoke } from '@tauri-apps/api/core';

import type { DesktopBootstrap, DesktopWorkspaceSnapshot } from './types';

export async function desktopBootstrap(): Promise<DesktopBootstrap> {
  return invoke<DesktopBootstrap>('desktop_bootstrap');
}

export async function loadWorkspaceSnapshot(): Promise<DesktopWorkspaceSnapshot> {
  return invoke<DesktopWorkspaceSnapshot>('load_workspace_snapshot');
}
