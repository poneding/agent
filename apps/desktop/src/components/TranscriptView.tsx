import type { TranscriptEntry } from '../types';

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
          <p>{entry.content}</p>
          {entry.truncated ? <small>truncated</small> : null}
        </li>
      ))}
    </ol>
  );
}
