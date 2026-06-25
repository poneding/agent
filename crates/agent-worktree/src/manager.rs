use crate::{WorktreeError, WorktreeLease, WorktreeLeaseId, WorktreeLeaseState, WorktreeRequest};
use agent_types::Timestamp;

pub trait WorktreeManager {
    fn plan(
        &mut self,
        request: WorktreeRequest,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError>;

    fn activate(
        &mut self,
        lease_id: &WorktreeLeaseId,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError>;

    fn release(
        &mut self,
        lease_id: &WorktreeLeaseId,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError>;

    fn get(&self, lease_id: &WorktreeLeaseId) -> Option<&WorktreeLease>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryWorktreeManager {
    leases: Vec<WorktreeLease>,
    next_sequence: u64,
}

impl InMemoryWorktreeManager {
    fn next_lease_id(&mut self) -> WorktreeLeaseId {
        self.next_sequence += 1;
        WorktreeLeaseId::generated(self.next_sequence)
    }

    fn lease_index(&self, lease_id: &WorktreeLeaseId) -> Result<usize, WorktreeError> {
        self.leases
            .iter()
            .position(|lease| lease.id == *lease_id)
            .ok_or_else(|| WorktreeError::LeaseNotFound(lease_id.as_str().to_string()))
    }

    fn transition(
        &mut self,
        lease_id: &WorktreeLeaseId,
        to: WorktreeLeaseState,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError> {
        let index = self.lease_index(lease_id)?;
        let from = self.leases[index].state;
        if from.is_terminal() {
            return Err(WorktreeError::InvalidTransition(format!(
                "terminal worktree lease state {from:?} cannot transition"
            )));
        }
        if !is_allowed_transition(from, to) {
            return Err(WorktreeError::InvalidTransition(format!(
                "worktree lease cannot transition from {from:?} to {to:?}"
            )));
        }

        self.leases[index].state = to;
        self.leases[index].updated_at = now;
        Ok(self.leases[index].clone())
    }
}

impl WorktreeManager for InMemoryWorktreeManager {
    fn plan(
        &mut self,
        request: WorktreeRequest,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError> {
        let id = self.next_lease_id();
        let lease = WorktreeLease::planned(id, request, now);
        self.leases.push(lease.clone());
        Ok(lease)
    }

    fn activate(
        &mut self,
        lease_id: &WorktreeLeaseId,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError> {
        self.transition(lease_id, WorktreeLeaseState::Active, now)
    }

    fn release(
        &mut self,
        lease_id: &WorktreeLeaseId,
        now: Timestamp,
    ) -> Result<WorktreeLease, WorktreeError> {
        self.transition(lease_id, WorktreeLeaseState::Released, now)
    }

    fn get(&self, lease_id: &WorktreeLeaseId) -> Option<&WorktreeLease> {
        self.leases.iter().find(|lease| lease.id == *lease_id)
    }
}

fn is_allowed_transition(from: WorktreeLeaseState, to: WorktreeLeaseState) -> bool {
    match from {
        WorktreeLeaseState::Planned => {
            matches!(
                to,
                WorktreeLeaseState::Active | WorktreeLeaseState::Released
            )
        }
        WorktreeLeaseState::Active => matches!(to, WorktreeLeaseState::Released),
        WorktreeLeaseState::Released | WorktreeLeaseState::CleanupFailed => false,
    }
}
