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
  );
}
