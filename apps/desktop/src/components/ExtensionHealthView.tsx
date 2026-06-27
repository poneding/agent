import type { UiExtensionSnapshot } from '../types';

export function ExtensionHealthView({ snapshot }: { snapshot: UiExtensionSnapshot }) {
  if (snapshot.extensions.length === 0) {
    return <div className="empty-list">No extensions</div>;
  }

  return (
    <div className="extension-health-view" aria-label="Extension health">
      <section className="settings-section">
        <h2>Extensions</h2>
        <ul className="provider-list">
          {snapshot.extensions.map((extension) => (
            <li key={extension.id}>
              <span>{extension.id}</span>
              <span>{extension.health}</span>
              <small>
                {extension.kind} / {extension.enabled ? 'enabled' : 'disabled'} / capabilities{' '}
                {extension.capability_count}
              </small>
            </li>
          ))}
        </ul>
      </section>

      <section className="settings-section">
        <h2>Diagnostics</h2>
        {snapshot.diagnostics.length === 0 ? (
          <p>No diagnostics</p>
        ) : (
          <ul>
            {snapshot.diagnostics.map((diagnostic) => (
              <li key={`${diagnostic.extension_id ?? 'global'}-${diagnostic.code}`}>
                <strong>{diagnostic.code}</strong>
                <span>{diagnostic.message}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="settings-section">
        <h2>Capability Reviews</h2>
        {snapshot.capability_reviews.length === 0 ? (
          <p>No reviews</p>
        ) : (
          <ul>
            {snapshot.capability_reviews.map((review) => (
              <li key={`${review.extension_id}-${review.capability_kind}-${review.resource}`}>
                <strong>{review.capability_kind}</strong>
                <small>
                  {review.extension_id} / {review.resource}
                </small>
                <span>{review.reason}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
