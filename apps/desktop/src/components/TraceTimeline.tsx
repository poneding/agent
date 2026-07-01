import type { TraceTimeline as TraceTimelineModel } from '../types';

export function TraceTimeline({
  timeline,
  error,
}: {
  timeline: TraceTimelineModel | null;
  error: string | null;
}) {
  if (error !== null) {
    return (
      <section className="diagnostic diagnostic-error" role="alert">
        <strong>Trace unavailable</strong>
        <span>{error}</span>
      </section>
    );
  }

  if (timeline === null || timeline.items.length === 0) {
    return <div className="empty-list">No trace events</div>;
  }

  return (
    <div className="trace-log-surface">
      <ol className="trace-timeline" aria-label="Trace timeline">
        {timeline.items.map((item) => (
          <li className={`trace-item trace-${item.severity}`} key={`${item.seq}-${item.kind}`}>
            <div className="trace-item-header">
              <span>{item.title}</span>
              <mark>{item.severity}</mark>
            </div>
            <p>{item.summary}</p>
            <small>
              #{item.seq} {item.kind}
            </small>
          </li>
        ))}
      </ol>
      <section className="runtime-log" aria-label="Runtime logs">
        <h2>Runtime logs</h2>
        {timeline.logs.length === 0 ? (
          <div className="empty-list">No runtime logs</div>
        ) : (
          <ol>
            {timeline.logs.map((entry) => (
              <li className={`trace-item trace-${entry.severity}`} key={entry.sequence}>
                <div className="trace-item-header">
                  <span>{entry.source}</span>
                  <mark>{entry.severity}</mark>
                </div>
                <p>{entry.message}</p>
                <small>
                  #{entry.sequence}
                  {entry.truncated ? ' truncated' : ''}
                </small>
              </li>
            ))}
          </ol>
        )}
      </section>
    </div>
  );
}
