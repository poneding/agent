import { FormEvent, useState } from 'react';

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
      <textarea
        aria-label="Message composer"
        disabled={disabled}
        placeholder="Type a message"
        rows={3}
        value={value}
        onChange={(event) => setValue(event.currentTarget.value)}
      />
      <button disabled={disabled || value.trim().length === 0} type="submit">
        Send
      </button>
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
