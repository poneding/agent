import type {
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopSettings,
  SettingsUiState,
} from '../types';

export function defaultDesktopSettings(): DesktopSettings {
  return {
    model: {
      provider: 'default',
      model: 'default',
    },
    policy: {
      default_decision: 'ask',
      require_approval_for_shell: true,
    },
    memory: {
      project_memory_enabled: false,
      long_term_memory_enabled: false,
    },
    mcp: {
      enabled: false,
      server_count: 0,
    },
    ui: {
      theme: 'system',
      show_trace_timeline: true,
    },
  };
}

export function createSettingsState(
  snapshot: DesktopConfigSnapshot | null = null,
  settings: DesktopSettings = defaultDesktopSettings(),
): SettingsUiState {
  return {
    snapshot,
    settings,
    validationDiagnostics: [],
    loading: false,
    error: null,
  };
}

export function reduceSettingsSnapshotLoaded(
  state: SettingsUiState,
  snapshot: DesktopConfigSnapshot,
): SettingsUiState {
  return {
    ...state,
    snapshot,
    settings: {
      ...state.settings,
      model: {
        provider: snapshot.snapshot.selected_provider,
        model: snapshot.snapshot.selected_model,
      },
    },
    loading: false,
    error: null,
  };
}

export function reduceSettingsValidation(
  state: SettingsUiState,
  diagnostics: DesktopBootstrapDiagnostic[],
): SettingsUiState {
  return {
    ...state,
    validationDiagnostics: diagnostics,
    loading: false,
  };
}

export function reduceSettingsError(state: SettingsUiState, error: string): SettingsUiState {
  return {
    ...state,
    loading: false,
    error,
  };
}
