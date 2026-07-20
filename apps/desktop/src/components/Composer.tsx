import { FormEvent, useState } from 'react';
import { ArrowUp, BadgeDollarSign, GitBranch, Square } from 'lucide-react';

import type { TurnSubmission } from '../types';

export function Composer({
  disabled,
  onSubmitPrompt,
  submission = null,
}: {
  disabled: boolean;
  onSubmitPrompt: (prompt: string) => void;
  submission?: TurnSubmission | null;
}) {
  const [value, setValue] = useState('');

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const prompt = value.trim();
    if (disabled || prompt.length === 0) {
      return;
    }
    onSubmitPrompt(prompt);
    setValue('');
  }

  return (
    <form className="composer" aria-label="Composer" onSubmit={handleSubmit}>
      <div className="composer-toolbar" aria-label="Composer options">
        <div className="segmented-control" aria-label="Execution mode">
          <button aria-pressed="true" type="button">
            Local
          </button>
          <button aria-pressed="false" type="button">
            <GitBranch aria-hidden="true" size={14} />
            Worktree
          </button>
        </div>
        <button className="ghost-control" type="button">
          <BadgeDollarSign aria-hidden="true" size={14} />
          Skills
        </button>
      </div>
      <div className="composer-input-shell">
        <textarea
          aria-label="Message composer"
          disabled={disabled}
          placeholder="Ask agent to inspect, edit, test, or explain this workspace..."
          rows={4}
          value={value}
          onChange={(event) => setValue(event.currentTarget.value)}
        />
        <button
          aria-label={submission?.status === 'submitting' ? 'Stop' : 'Send'}
          className="send-button"
          disabled={disabled || value.trim().length === 0}
          type="submit"
        >
          {submission?.status === 'submitting' ? (
            <Square aria-hidden="true" size={15} />
          ) : (
            <ArrowUp aria-hidden="true" size={16} />
          )}
          <span>{submission?.status === 'submitting' ? 'Stop' : 'Send'}</span>
        </button>
      </div>
      {submission === null ? null : (
        <div className="composer-status" role="status">
          <strong>{submission.status}</strong>
          <span>{submission.prompt_preview}</span>
          {submission.ack === null ? null : (
            <small>{submission.ack.pending_frame_count} pending frames</small>
          )}
          {submission.error === null ? null : (
            <p className="composer-error" role="alert">
              {submission.error.code}: {submission.error.message}
            </p>
          )}
        </div>
      )}
    </form>
  );
}
