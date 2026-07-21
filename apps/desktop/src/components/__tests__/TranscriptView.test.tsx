import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { TranscriptView } from '../TranscriptView';

describe('TranscriptView markdown', () => {
  it('renders assistant content as markdown elements', () => {
    render(
      <TranscriptView
        entries={[
          {
            id: 'a1',
            kind: 'assistant',
            content: '# Hello\n\nUse `code` and **bold**.\n\n- one\n- two\n',
            truncated: false,
          },
        ]}
      />,
    );

    expect(screen.getByRole('heading', { level: 1, name: 'Hello' })).toBeTruthy();
    expect(screen.getByText('code')).toBeTruthy();
    expect(screen.getByText('bold').tagName).toBe('STRONG');
    expect(screen.getByText('one')).toBeTruthy();
  });

  it('keeps user content as plain text', () => {
    const { container } = render(
      <TranscriptView
        entries={[
          {
            id: 'u1',
            kind: 'user',
            content: '**not markdown**',
            truncated: false,
          },
        ]}
      />,
    );

    expect(screen.getByText('**not markdown**')).toBeTruthy();
    expect(container.querySelector('.markdown-body')).toBeNull();
    expect(container.querySelector('h1')).toBeNull();
  });
});
