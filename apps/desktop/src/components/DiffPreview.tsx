import type { UiApprovalPreviewSummary } from '../types';

export function DiffPreview({ preview }: { preview: UiApprovalPreviewSummary }) {
  const lines = preview.summary.split('\n').filter((line) => line.length > 0);

  return (
    <section className="diff-preview" aria-label="Diff preview">
      <div className="preview-meta">
        <strong>{preview.kind}</strong>
        <span>{preview.resource}</span>
        {preview.redacted ? <mark>redacted</mark> : null}
        {preview.truncated ? <mark>truncated</mark> : null}
      </div>
      <pre>
        {lines.map((line) => (
          <code key={line}>{line}</code>
        ))}
      </pre>
    </section>
  );
}
