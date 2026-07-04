import type { DesktopDiffReview, UiApprovalPreviewSummary } from '../types';

export function DiffPreview({
  preview,
  review = null,
}: {
  preview: UiApprovalPreviewSummary;
  review?: DesktopDiffReview | null;
}) {
  const lines = preview.summary.split('\n').filter((line) => line.length > 0);
  const selectedFile =
    review?.files.find((file) => file.path === review.selected_path) ?? review?.files[0] ?? null;

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
      {review === null ? null : (
        <div className="visual-diff-review" aria-label="Visual diff review">
          <ul className="diff-file-list">
            {review.files.map((file) => (
              <li key={file.path}>
                <strong>{file.path}</strong>
                <span>
                  +{file.added_lines} -{file.removed_lines}
                </span>
                <small>{file.summary}</small>
              </li>
            ))}
          </ul>
          {selectedFile === null ? null : (
            <div className="diff-hunks">
              {selectedFile.hunks.map((hunk) => (
                <section
                  className="diff-hunk"
                  key={`${selectedFile.path}-${hunk.old_start}-${hunk.new_start}`}
                >
                  <header>
                    @@ -{hunk.old_start} +{hunk.new_start} @@
                    {hunk.truncated ? <mark>truncated</mark> : null}
                  </header>
                  <pre>
                    {hunk.lines.map((line, index) => (
                      <code
                        className={`diff-line diff-line-${line.kind}`}
                        key={`${line.kind}-${line.old_line ?? 'n'}-${line.new_line ?? 'n'}-${index}`}
                      >
                        {line.content}
                        {line.truncated ? ' [truncated]' : ''}
                      </code>
                    ))}
                  </pre>
                </section>
              ))}
            </div>
          )}
          {review.disabled_reason === null || review.disabled_reason === undefined ? null : (
            <p className="approval-disabled">{review.disabled_reason}</p>
          )}
        </div>
      )}
    </section>
  );
}
