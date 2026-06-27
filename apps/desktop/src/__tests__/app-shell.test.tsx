import '@testing-library/jest-dom/vitest';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { AppShell, BootstrapError } from '../App';
import type { DesktopBootstrap } from '../types';

const bootstrap: DesktopBootstrap = {
  app_name: 'agent',
  app_version: '0.1.0',
  runtime_state: 'not_started',
  route: 'conversation',
  diagnostics: [
    {
      severity: 'warning',
      code: 'desktop.bootstrap',
      message: 'using fixture bootstrap',
    },
  ],
};

describe('AppShell', () => {
  it('renders the desktop shell regions from bootstrap state', () => {
    render(<AppShell bootstrap={bootstrap} bootstrapError={null} />);

    expect(screen.getByRole('navigation', { name: 'Desktop navigation' })).toBeInTheDocument();
    expect(screen.getByRole('main', { name: 'Conversation workspace' })).toBeInTheDocument();
    expect(screen.getByRole('complementary', { name: 'Context panel' })).toBeInTheDocument();
    expect(screen.getAllByText('agent')).toHaveLength(2);
    expect(screen.getAllByText('0.1.0')).toHaveLength(2);
    expect(screen.getAllByText('not_started')).toHaveLength(2);
    expect(screen.getByText('desktop.bootstrap')).toBeInTheDocument();
  });

  it('renders bootstrap errors as recoverable diagnostics', () => {
    render(<BootstrapError message="bootstrap failed" />);

    expect(screen.getByRole('alert')).toHaveTextContent('bootstrap failed');
  });
});
