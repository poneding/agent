import type { UiMemorySnapshot } from '../types';

export function MemoryView({ snapshot }: { snapshot: UiMemorySnapshot | null }) {
  if (snapshot === null) {
    return <section className="memory-view empty-list">Memory unavailable</section>;
  }

  const retrieval = snapshot.retrieval;
  const influence = snapshot.influence;

  return (
    <section className="memory-view" aria-label="Memory">
      <div className="memory-view-header">
        <div>
          <h2>Memory</h2>
          <p>{snapshot.scope_label}</p>
        </div>
        <span className={snapshot.enabled ? 'memory-state enabled' : 'memory-state disabled'}>
          {snapshot.enabled ? 'enabled' : 'disabled'}
        </span>
      </div>

      <dl className="memory-grid">
        <dt>Layers</dt>
        <dd>{snapshot.layer_count}</dd>
        <dt>Records</dt>
        <dd>{snapshot.record_count}</dd>
        <dt>Proposals</dt>
        <dd>{snapshot.proposal_count}</dd>
        <dt>Selected</dt>
        <dd>{snapshot.selected_count}</dd>
        <dt>Rejected</dt>
        <dd>{snapshot.rejected_count}</dd>
        <dt>Tombstones</dt>
        <dd>{snapshot.tombstone_count}</dd>
      </dl>

      <div className="memory-retention" aria-label="Memory retention">
        <span>{snapshot.retention.memory_enabled ? 'retention on' : 'retention off'}</span>
        <span>
          {snapshot.retention.export_includes_sensitive
            ? 'sensitive export allowed'
            : 'sensitive export excluded'}
        </span>
        <span>
          {snapshot.retention.sensitive_memory_requires_review
            ? 'sensitive review required'
            : 'sensitive review optional'}
        </span>
      </div>

      {snapshot.records.length === 0 ? (
        <p className="empty-list">No memory records</p>
      ) : (
        <ul className="memory-list" aria-label="Memory records">
          {snapshot.records.map((record) => (
            <li key={record.id}>
              <div>
                <strong>{record.id}</strong>
                <span>{record.text_preview ?? '[hidden]'}</span>
              </div>
              <span>{record.sensitivity}</span>
            </li>
          ))}
        </ul>
      )}

      {snapshot.proposals.length === 0 ? (
        <p className="empty-list">No memory proposals</p>
      ) : (
        <ul className="memory-list" aria-label="Memory proposals">
          {snapshot.proposals.map((proposal) => (
            <li key={proposal.proposal_id}>
              <div>
                <strong>{proposal.proposal_id}</strong>
                <span>{proposal.summary}</span>
              </div>
              <span>{proposal.status}</span>
            </li>
          ))}
        </ul>
      )}

      {retrieval === null || retrieval === undefined ? null : (
        <section className="memory-subsection" aria-label="Memory retrieval">
          <h3>Retrieval</h3>
          <p>
            {retrieval.selected_count} selected, {retrieval.rejected_count} rejected
          </p>
          <ul className="memory-list compact">
            {retrieval.candidates.map((candidate) => (
              <li key={`${candidate.memory_id}:${candidate.rank}`}>
                <div>
                  <strong>{candidate.memory_id}</strong>
                  <span>{candidate.preview ?? '[hidden]'}</span>
                </div>
                <span>{candidate.selected ? 'selected' : 'rejected'}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {influence === null || influence === undefined ? null : (
        <section className="memory-subsection" aria-label="Memory influence">
          <h3>Influence</h3>
          <p>
            {influence.selected_count} selected, {influence.rejected_count} rejected
          </p>
        </section>
      )}

      {snapshot.diagnostics.length === 0 ? null : (
        <ul className="memory-diagnostics" aria-label="Memory diagnostics">
          {snapshot.diagnostics.map((diagnostic) => (
            <li key={`${diagnostic.code}:${diagnostic.message}`}>
              <strong>{diagnostic.severity}</strong>
              <span>{diagnostic.code}</span>
              <span>{diagnostic.message}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
