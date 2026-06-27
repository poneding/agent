import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { SettingsView } from '../components/SettingsView';
import { TraceTimeline } from '../components/TraceTimeline';
import { createSettingsState, reduceSettingsValidation } from '../state/settings';
import { createTraceState } from '../state/trace';
import type {
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopSettings,
  TraceTimeline as TraceTimelineModel,
} from '../types';

const trace: TraceTimelineModel = {
  conversation_id: 'conversation-1',
  items: [
    {
      seq: 1,
      kind: 'checkpoint',
      title: 'Checkpoint',
      summary: 'workspace saved',
      severity: 'info',
    },
    {
      seq: 2,
      kind: 'warning_policy',
      title: 'Warning Policy',
      summary: 'policy warning',
      severity: 'warning',
    },
    {
      seq: 3,
      kind: 'tool_error',
      title: 'Tool Error',
      summary: 'failed with [redacted]',
      severity: 'error',
    },
  ],
};

const settings: DesktopSettings = {
  model: {
    provider: 'fake',
    model: 'fake-scripted',
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

const snapshot: DesktopConfigSnapshot = {
  snapshot: {
    selected_provider: 'fake',
    selected_model: 'fake-scripted',
    providers: [
      {
        id: 'fake',
        kind: 'fake',
        default_model: 'fake-scripted',
        enabled: true,
      },
    ],
    diagnostics: [
      {
        severity: 'warning',
        code: 'config.fixture',
        message: 'using fake provider',
        field_path: 'model.provider',
        source: 'test',
      },
    ],
  },
};

describe('trace and settings UI', () => {
  afterEach(() => cleanup());

  it('creates trace state and renders an empty trace', () => {
    const state = createTraceState('conversation-1');

    render(<TraceTimeline timeline={state.timeline} error={null} />);

    expect(screen.getByText('No trace events')).toBeInTheDocument();
  });

  it('renders trace severity and redacted summaries', () => {
    render(<TraceTimeline timeline={trace} error={null} />);

    expect(screen.getByText('Checkpoint')).toBeInTheDocument();
    expect(screen.getByText('warning')).toBeInTheDocument();
    expect(screen.getByText('error')).toBeInTheDocument();
    expect(screen.getByText('failed with [redacted]')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();
  });

  it('renders settings snapshot fields and provider list', () => {
    render(<SettingsView snapshot={snapshot} settings={settings} diagnostics={[]} error={null} />);

    expect(screen.getByText('fake-scripted')).toBeInTheDocument();
    expect(screen.getByText('ask')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Model providers')).getByText('fake')).toBeInTheDocument();
    expect(screen.getByText('config.fixture')).toBeInTheDocument();
  });

  it('reduces and renders invalid settings diagnostics', () => {
    const diagnostics: DesktopBootstrapDiagnostic[] = [
      {
        severity: 'error',
        code: 'desktop.settings.invalid',
        message: 'model provider cannot be empty',
      },
    ];
    const state = reduceSettingsValidation(createSettingsState(snapshot, settings), diagnostics);

    render(
      <SettingsView
        snapshot={state.snapshot}
        settings={state.settings}
        diagnostics={state.validationDiagnostics}
        error={null}
      />,
    );

    expect(screen.getByRole('alert')).toHaveTextContent('model provider cannot be empty');
  });

  it('does not render raw secret-like settings diagnostics', () => {
    const redactedSnapshot: DesktopConfigSnapshot = {
      snapshot: {
        ...snapshot.snapshot,
        diagnostics: [
          {
            severity: 'warning',
            code: 'config.secret',
            message: '[redacted] [redacted] should not leak',
            field_path: 'model.provider',
            source: 'test',
          },
        ],
      },
    };

    render(
      <SettingsView
        snapshot={redactedSnapshot}
        settings={settings}
        diagnostics={[]}
        error={null}
      />,
    );

    expect(screen.getByText('[redacted] [redacted] should not leak')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();
    expect(screen.queryByText('api_key')).not.toBeInTheDocument();
  });
});
