import type { UiTeamWorkflowSnapshot } from '../types';

export function TeamWorkflowView({ snapshot }: { snapshot: UiTeamWorkflowSnapshot | null }) {
  if (snapshot === null) {
    return <section className="team-workflow empty-list">Team workflow unavailable</section>;
  }

  const workflow = snapshot.snapshot;
  const capabilityText = `${workflow.capability_ceiling.grant_count} grants, ${
    workflow.capability_ceiling.policy_blocked ? 'policy blocked' : 'policy clear'
  }`;
  const mergeReviewText = `${workflow.merge_review.approving_review_count}/${workflow.merge_review.required_review_count} approvals, ${workflow.merge_review.blocking_review_count} blocks, ${workflow.merge_review.artifact_count} artifacts`;
  const blackboardText = `${workflow.blackboard.entry_count} entries, ${workflow.blackboard.approve_count} approvals, ${workflow.blackboard.request_changes_count} changes, ${workflow.blackboard.block_count} blocks`;

  return (
    <section className="team-workflow" aria-label="Team workflow">
      <div className="team-workflow-header">
        <div>
          <h2>Team Workflow</h2>
          <p>{workflow.task_id}</p>
        </div>
        <span className={`team-readiness ${workflow.readiness}`}>{workflow.readiness}</span>
      </div>
      <dl className="team-workflow-grid">
        <dt>Operation</dt>
        <dd>{snapshot.operation}</dd>
        <dt>Status</dt>
        <dd>{workflow.status}</dd>
        <dt>Dry Run</dt>
        <dd>{snapshot.dry_run ? 'yes' : 'no'}</dd>
        <dt>Pending Approvals</dt>
        <dd>{workflow.pending_approval_count}</dd>
        <dt>Capability Ceiling</dt>
        <dd>{capabilityText}</dd>
        <dt>Merge Review</dt>
        <dd>{mergeReviewText}</dd>
        <dt>Blackboard</dt>
        <dd>{blackboardText}</dd>
      </dl>
      {workflow.handoffs.length > 0 ? (
        <ul className="team-workflow-list" aria-label="Team handoffs">
          {workflow.handoffs.map((handoff) => (
            <li key={`${handoff.subagent_id}:${handoff.worktree_lease_id}`}>
              <span>{handoff.role}</span>
              <span>{handoff.status}</span>
              <span>{handoff.objective}</span>
            </li>
          ))}
        </ul>
      ) : null}
      {workflow.conflicts.length > 0 ? (
        <ul className="team-workflow-list" aria-label="Team conflicts">
          {workflow.conflicts.map((conflict) => (
            <li key={`${conflict.path}:${conflict.kind}`}>
              <span>{conflict.path}</span>
              <span>{conflict.kind}</span>
              <span>{conflict.next_action}</span>
            </li>
          ))}
        </ul>
      ) : null}
      {workflow.diagnostics.length > 0 || snapshot.errors.length > 0 ? (
        <ul className="team-workflow-diagnostics" aria-label="Team diagnostics">
          {workflow.diagnostics.map((diagnostic) => (
            <li key={`${diagnostic.code}:${diagnostic.message}`}>
              <strong>{diagnostic.severity}</strong>
              <span>{diagnostic.code}</span>
              <span>{diagnostic.message}</span>
            </li>
          ))}
          {snapshot.errors.map((error) => (
            <li key={`${error.code}:${error.message}`}>
              <strong>{error.recoverable ? 'recoverable' : 'fatal'}</strong>
              <span>{error.code}</span>
              <span>{error.message}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
