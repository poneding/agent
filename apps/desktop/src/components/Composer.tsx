import { FormEvent, useState } from 'react';

export function Composer({
  disabled,
  onSubmitPrompt,
}: {
  disabled: boolean;
  onSubmitPrompt: (prompt: string) => void;
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
    </form>
  );
}
