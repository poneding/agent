import type { TranscriptEntry } from '../types';

export function TranscriptView({ entries }: { entries: TranscriptEntry[] }) {
  if (entries.length === 0) {
    return <div className="empty-transcript">No messages yet</div>;
  }

  return (
    <ol className="transcript" aria-label="Transcript">
      {entries.map((entry) => (
        <li className={`transcript-entry transcript-entry-${entry.kind}`} key={entry.id}>
          <span>{entry.kind}</span>
          <p>{entry.content}</p>
        </li>
      ))}
    </ol>
  );
}
