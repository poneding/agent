import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { SettingsView } from '../components/SettingsView';
import { TaskDetail } from '../components/TaskDetail';
import { TaskList } from '../components/TaskList';
import { TraceTimeline } from '../components/TraceTimeline';
import { createSettingsState, reduceSettingsValidation } from '../state/settings';
import { buildTaskMonitor } from '../state/tasks';
import { createTraceState } from '../state/trace';
import type {
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopSettings,
  DesktopTaskItem,
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
  logs: [
    {
      sequence: 1,
      severity: 'info',
      source: 'checkpoint',
      message: 'workspace saved',
      truncated: false,
    },
    {
      sequence: 3,
      severity: 'error',
      source: 'tool_error',
      message: 'failed with [redacted]',
      truncated: true,
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
        capabilities: {
          streaming: true,
          tool_calls: true,
          structured_outputs: false,
          image_input: false,
          max_context_tokens: 4096,
          max_output_tokens: 1024,
        },
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

const tasks: DesktopTaskItem[] = [
  {
    task_id: 'task-1',
    workspace_id: 'workspace-1',
    conversation_id: 'conversation-1',
    state: 'running',
    summary: 'Run verification',
    updated_at: '2026-06-27T00:00:00Z',
    latest_run_summary: 'cargo test running',
    artifact_count: 2,
  },
  {
    task_id: 'task-2',
    workspace_id: 'workspace-1',
    conversation_id: 'conversation-1',
    state: 'waiting_approval',
    summary: 'Approve command',
    updated_at: '2026-06-27T00:01:00Z',
    latest_run_summary: null,
    artifact_count: 0,
  },
  {
    task_id: 'task-3',
    workspace_id: 'workspace-1',
    conversation_id: 'conversation-1',
    state: 'failed',
    summary: 'Failed task',
    updated_at: '2026-06-27T00:02:00Z',
    latest_run_summary: 'failed with [redacted]',
    artifact_count: 1,
  },
];

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
    expect(screen.getAllByText('error')).toHaveLength(2);
    expect(screen.getAllByText('failed with [redacted]')).toHaveLength(2);
    expect(screen.getByLabelText('Runtime logs')).toBeInTheDocument();
    expect(screen.getByText('tool_error')).toBeInTheDocument();
    expect(screen.getByText('#3 truncated')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();
  });

  it('renders settings snapshot fields and provider list', () => {
    render(<SettingsView snapshot={snapshot} settings={settings} diagnostics={[]} error={null} />);

    expect(screen.getByText('fake-scripted')).toBeInTheDocument();
    expect(screen.getByText('ask')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Model providers')).getByText('fake')).toBeInTheDocument();
    expect(screen.getByText('streaming yes / tools yes / context 4096')).toBeInTheDocument();
    expect(screen.getByText('config.fixture')).toBeInTheDocument();
  });

  it('renders task monitor counts and selected task details', () => {
    const monitor = buildTaskMonitor(tasks);

    render(
      <>
        <TaskList tasks={monitor.tasks} activeTaskId="task-1" onSelect={() => undefined} />
        <TaskDetail task={monitor.tasks[0]} />
      </>,
    );

    expect(within(screen.getByLabelText('Task monitor counts')).getByText('3')).toBeInTheDocument();
    expect(screen.getByText('Approve command')).toBeInTheDocument();
    expect(screen.getByText('cargo test running')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(monitor.counts.active).toBe(1);
    expect(monitor.counts.waiting_approval).toBe(1);
    expect(monitor.counts.failed).toBe(1);
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
