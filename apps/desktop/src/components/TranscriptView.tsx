import type { TranscriptEntry } from '../types';
import { MarkdownBody } from './MarkdownBody';

function usesMarkdown(kind: TranscriptEntry['kind']): boolean {
  // Align with TUI: model/assistant (and reasoning-style system notes if any)
  // get Markdown; user/diagnostic stay plain.
  return kind === 'assistant';
}

export function TranscriptView({ entries }: { entries: TranscriptEntry[] }) {
  if (entries.length === 0) {
    return (
      <div className="empty-transcript">
        <strong>Start with the workspace in view.</strong>
        <span>Ask for an edit, a review, a command run, or a plan.</span>
      </div>
    );
  }

  return (
    <ol className="transcript" aria-label="Transcript">
      {entries.map((entry) => (
        <li className={`transcript-entry transcript-entry-${entry.kind}`} key={entry.id}>
          <header>
            <span>{entry.kind}</span>
          </header>
          {usesMarkdown(entry.kind) ? (
            <MarkdownBody content={entry.content} />
          ) : (
            <p>{entry.content}</p>
          )}
          {entry.truncated ? <small>truncated</small> : null}
        </li>
      ))}
    </ol>
  );
}
