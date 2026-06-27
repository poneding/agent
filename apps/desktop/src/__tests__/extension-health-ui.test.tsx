import '@testing-library/jest-dom/vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { ExtensionHealthView } from '../components/ExtensionHealthView';
import type { UiExtensionSnapshot } from '../types';

const snapshot: UiExtensionSnapshot = {
  extensions: [
    {
      id: 'hello-plugin',
      kind: 'plugin',
      enabled: true,
      health: 'healthy',
      version: '0.1.0',
      source_label: 'plugins/hello',
      capability_count: 1,
    },
    {
      id: 'mcp-filesystem',
      kind: 'mcp_server',
      enabled: false,
      health: 'unhealthy',
      version: null,
      source_label: 'mcp filesystem',
      capability_count: 0,
    },
  ],
  capability_reviews: [
    {
      extension_id: 'hello-plugin',
      capability_kind: 'tool',
      resource: 'fs.read',
      reason: 'capability review is required',
    },
  ],
  diagnostics: [
    {
      severity: 'error',
      code: 'extension.unhealthy',
      message: 'server failed with [redacted]',
      extension_id: 'mcp-filesystem',
      truncated: false,
    },
  ],
};

describe('extension health UI', () => {
  afterEach(() => cleanup());

  it('renders extension health, diagnostics, and capability reviews', () => {
    render(<ExtensionHealthView snapshot={snapshot} />);

    expect(screen.getByText('hello-plugin')).toBeInTheDocument();
    expect(screen.getByText('mcp-filesystem')).toBeInTheDocument();
    expect(screen.getByText('unhealthy')).toBeInTheDocument();
    expect(screen.getByText('extension.unhealthy')).toBeInTheDocument();
    expect(screen.getByText('capability review is required')).toBeInTheDocument();
    expect(within(screen.getByLabelText('Extension health')).getByText('healthy')).toBeInTheDocument();
    expect(screen.queryByText('sk-secret')).not.toBeInTheDocument();
  });

  it('renders empty state', () => {
    render(
      <ExtensionHealthView
        snapshot={{ extensions: [], capability_reviews: [], diagnostics: [] }}
      />,
    );

    expect(screen.getByText('No extensions')).toBeInTheDocument();
  });
});
