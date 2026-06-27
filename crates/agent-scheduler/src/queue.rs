use agent_types::{TaskId, TaskState, Timestamp};
use time::Duration;

use crate::{
    LeaseStatus, QuotaDecision, QuotaPolicy, QuotaSnapshot, RetryDecision, RetryPolicy,
    SchedulerError, TaskLease, TaskLeaseId, TaskPriority, TaskRun, TaskTransition,
    evaluate_task_quota, transition_task_state,
};

pub trait TaskQueue {
    fn get_task(&self, task_id: TaskId) -> Option<&TaskRun>;

    fn enqueue(&mut self, run: TaskRun) -> Result<(), SchedulerError>;

    fn acquire_next(
        &mut self,
        worker_id: &str,
        now: Timestamp,
    ) -> Result<Option<TaskLease>, SchedulerError>;

    fn acquire_next_with_quota(
        &mut self,
        worker_id: &str,
        now: Timestamp,
        policy: &QuotaPolicy,
        snapshot: &QuotaSnapshot,
    ) -> Result<Option<TaskLease>, SchedulerError>;

    fn complete(
        &mut self,
        lease_id: &TaskLeaseId,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError>;

    fn fail(
        &mut self,
        lease_id: &TaskLeaseId,
        error: String,
        now: Timestamp,
    ) -> Result<RetryDecision, SchedulerError>;

    fn cancel(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError>;

    fn pause(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError>;

    fn resume(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError>;

    fn release_expired_leases(&mut self, now: Timestamp) -> Vec<TaskLease>;
}

#[derive(Clone, Debug)]
pub struct InMemoryTaskQueue {
    retry_policy: RetryPolicy,
    lease_duration_ms: u64,
    tasks: Vec<QueueEntry>,
    leases: Vec<TaskLease>,
    next_lease_sequence: u64,
}

impl InMemoryTaskQueue {
    pub fn new(retry_policy: RetryPolicy, lease_duration_ms: u64) -> Result<Self, SchedulerError> {
        let retry_policy = retry_policy.validate()?;
        if lease_duration_ms == 0 {
            return Err(SchedulerError::InvalidLease(
                "lease_duration_ms must be at least 1".to_string(),
            ));
        }

        Ok(Self {
            retry_policy,
            lease_duration_ms,
            tasks: Vec::new(),
            leases: Vec::new(),
            next_lease_sequence: 1,
        })
    }

    fn wake_ready_sleeping_tasks(&mut self, now: Timestamp) -> Result<(), SchedulerError> {
        for entry in &mut self.tasks {
            if entry.run.state == TaskState::Sleeping
                && entry.active_lease.is_none()
                && entry.available_at <= now
            {
                entry.run = transition_task_state(
                    &entry.run,
                    TaskTransition {
                        from: TaskState::Sleeping,
                        to: TaskState::Queued,
                        reason: "retry time reached".to_string(),
                        at: now,
                    },
                )?;
            }
        }
        Ok(())
    }

    fn ready_task_indices(&self, now: Timestamp) -> Vec<usize> {
        let mut indices = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.run.state == TaskState::Queued
                    && entry.active_lease.is_none()
                    && entry.available_at <= now
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        indices.sort_by(|left, right| {
            let left = &self.tasks[*left];
            let right = &self.tasks[*right];
            priority_rank(left.run.priority)
                .cmp(&priority_rank(right.run.priority))
                .then_with(|| left.run.created_at.cmp(&right.run.created_at))
                .then_with(|| left.run.task_id.cmp(&right.run.task_id))
        });
        indices
    }

    fn validate_worker_id(worker_id: &str) -> Result<(), SchedulerError> {
        if worker_id.trim().is_empty() {
            return Err(SchedulerError::InvalidLease(
                "worker_id cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn lease_task_at_index(
        &mut self,
        task_index: usize,
        worker_id: &str,
        now: Timestamp,
    ) -> Result<TaskLease, SchedulerError> {
        let task_id = self.tasks[task_index].run.task_id;
        let next_run = transition_task_state(
            &self.tasks[task_index].run,
            TaskTransition {
                from: TaskState::Queued,
                to: TaskState::Running,
                reason: "lease acquired".to_string(),
                at: now,
            },
        )?;
        let lease = self.build_lease(task_id, worker_id, now)?;

        self.tasks[task_index].run = next_run;
        self.tasks[task_index].active_lease = Some(lease.id.clone());
        Ok(lease)
    }

    fn active_lease_index(&self, lease_id: &TaskLeaseId) -> Result<usize, SchedulerError> {
        self.leases
            .iter()
            .position(|lease| lease.id == *lease_id && lease.status == LeaseStatus::Active)
            .ok_or_else(|| SchedulerError::InvalidLease("active lease not found".to_string()))
    }

    fn task_index(&self, task_id: TaskId) -> Result<usize, SchedulerError> {
        self.tasks
            .iter()
            .position(|entry| entry.run.task_id == task_id)
            .ok_or_else(|| SchedulerError::InvalidTask(format!("task not found: {task_id}")))
    }

    fn mark_active_lease(&mut self, lease_id: &TaskLeaseId, status: LeaseStatus) {
        if let Some(lease) = self.leases.iter_mut().find(|lease| lease.id == *lease_id) {
            lease.status = status;
        }
    }

    fn build_lease(
        &mut self,
        task_id: TaskId,
        worker_id: &str,
        now: Timestamp,
    ) -> Result<TaskLease, SchedulerError> {
        let id = TaskLeaseId::generated(self.next_lease_sequence);
        self.next_lease_sequence += 1;
        let lease = TaskLease {
            id,
            task_id,
            worker_id: worker_id.to_string(),
            acquired_at: now,
            expires_at: add_millis(now, self.lease_duration_ms)?,
            status: LeaseStatus::Active,
        };
        self.leases.push(lease.clone());
        Ok(lease)
    }

    fn validate_task_can_retry(&self, run: &TaskRun, next_attempt: u32) -> bool {
        let max_attempts = run.max_attempts.min(self.retry_policy.max_attempts);
        next_attempt < max_attempts
    }
}

impl Default for InMemoryTaskQueue {
    fn default() -> Self {
        Self::new(RetryPolicy::default(), 1_000).expect("default retry policy and lease are valid")
    }
}

impl TaskQueue for InMemoryTaskQueue {
    fn get_task(&self, task_id: TaskId) -> Option<&TaskRun> {
        self.tasks
            .iter()
            .find(|entry| entry.run.task_id == task_id)
            .map(|entry| &entry.run)
    }

    fn enqueue(&mut self, run: TaskRun) -> Result<(), SchedulerError> {
        if self
            .tasks
            .iter()
            .any(|entry| entry.run.task_id == run.task_id)
        {
            return Err(SchedulerError::InvalidTask(format!(
                "task already exists: {}",
                run.task_id
            )));
        }
        if run.max_attempts == 0 {
            return Err(SchedulerError::InvalidTask(
                "task max_attempts must be at least 1".to_string(),
            ));
        }

        let queued = match run.state {
            TaskState::Created => transition_task_state(
                &run,
                TaskTransition {
                    from: TaskState::Created,
                    to: TaskState::Queued,
                    reason: "enqueue".to_string(),
                    at: run.created_at,
                },
            )?,
            TaskState::Queued => run,
            state => {
                return Err(SchedulerError::InvalidTask(format!(
                    "cannot enqueue task in state {state:?}"
                )));
            }
        };

        self.tasks.push(QueueEntry {
            available_at: queued.created_at,
            active_lease: None,
            run: queued,
        });
        Ok(())
    }

    fn acquire_next(
        &mut self,
        worker_id: &str,
        now: Timestamp,
    ) -> Result<Option<TaskLease>, SchedulerError> {
        Self::validate_worker_id(worker_id)?;

        self.wake_ready_sleeping_tasks(now)?;
        let Some(task_index) = self.ready_task_indices(now).into_iter().next() else {
            return Ok(None);
        };

        self.lease_task_at_index(task_index, worker_id, now)
            .map(Some)
    }

    fn acquire_next_with_quota(
        &mut self,
        worker_id: &str,
        now: Timestamp,
        policy: &QuotaPolicy,
        snapshot: &QuotaSnapshot,
    ) -> Result<Option<TaskLease>, SchedulerError> {
        Self::validate_worker_id(worker_id)?;
        self.wake_ready_sleeping_tasks(now)?;

        for task_index in self.ready_task_indices(now) {
            if evaluate_task_quota(policy, snapshot, &self.tasks[task_index].run)
                == QuotaDecision::Allow
            {
                return self
                    .lease_task_at_index(task_index, worker_id, now)
                    .map(Some);
            }
        }
        Ok(None)
    }

    fn complete(
        &mut self,
        lease_id: &TaskLeaseId,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError> {
        let lease_index = self.active_lease_index(lease_id)?;
        let task_id = self.leases[lease_index].task_id;
        let task_index = self.task_index(task_id)?;
        if self.tasks[task_index].active_lease.as_ref() != Some(lease_id) {
            return Err(SchedulerError::InvalidLease(
                "lease does not own task".to_string(),
            ));
        }

        let next = transition_task_state(
            &self.tasks[task_index].run,
            TaskTransition {
                from: TaskState::Running,
                to: TaskState::Succeeded,
                reason: "completed".to_string(),
                at: now,
            },
        )?;
        self.tasks[task_index].run = next.clone();
        self.tasks[task_index].active_lease = None;
        self.leases[lease_index].status = LeaseStatus::Completed;
        Ok(next)
    }

    fn fail(
        &mut self,
        lease_id: &TaskLeaseId,
        error: String,
        now: Timestamp,
    ) -> Result<RetryDecision, SchedulerError> {
        if error.trim().is_empty() {
            return Err(SchedulerError::InvalidTask(
                "failure error cannot be empty".to_string(),
            ));
        }

        let lease_index = self.active_lease_index(lease_id)?;
        let task_id = self.leases[lease_index].task_id;
        let task_index = self.task_index(task_id)?;
        if self.tasks[task_index].active_lease.as_ref() != Some(lease_id) {
            return Err(SchedulerError::InvalidLease(
                "lease does not own task".to_string(),
            ));
        }

        let mut running = self.tasks[task_index].run.clone();
        let next_attempt = running.attempt.saturating_add(1);
        running.attempt = next_attempt;
        self.tasks[task_index].active_lease = None;
        self.leases[lease_index].status = LeaseStatus::Failed;

        if self.validate_task_can_retry(&running, next_attempt) {
            let sleeping = transition_task_state(
                &running,
                TaskTransition {
                    from: TaskState::Running,
                    to: TaskState::Sleeping,
                    reason: error,
                    at: now,
                },
            )?;
            let run_at = add_millis(now, self.retry_policy.delay_ms_for_attempt(next_attempt))?;
            self.tasks[task_index].available_at = run_at;
            self.tasks[task_index].run = sleeping.clone();
            Ok(RetryDecision::RetryAt {
                task: sleeping,
                run_at,
            })
        } else {
            let failed = transition_task_state(
                &running,
                TaskTransition {
                    from: TaskState::Running,
                    to: TaskState::Failed,
                    reason: error.clone(),
                    at: now,
                },
            )?;
            self.tasks[task_index].available_at = now;
            self.tasks[task_index].run = failed.clone();
            Ok(RetryDecision::Exhausted {
                task: failed,
                error,
            })
        }
    }

    fn cancel(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError> {
        let task_index = self.task_index(task_id)?;
        let next = transition_task_state(
            &self.tasks[task_index].run,
            TaskTransition {
                from: self.tasks[task_index].run.state,
                to: TaskState::Cancelled,
                reason,
                at: now,
            },
        )?;

        if let Some(lease_id) = self.tasks[task_index].active_lease.clone() {
            self.mark_active_lease(&lease_id, LeaseStatus::Cancelled);
        }
        self.tasks[task_index].active_lease = None;
        self.tasks[task_index].available_at = now;
        self.tasks[task_index].run = next.clone();
        Ok(next)
    }

    fn pause(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError> {
        let task_index = self.task_index(task_id)?;
        let next = transition_task_state(
            &self.tasks[task_index].run,
            TaskTransition {
                from: TaskState::Running,
                to: TaskState::Paused,
                reason,
                at: now,
            },
        )?;
        if let Some(lease_id) = self.tasks[task_index].active_lease.clone() {
            self.mark_active_lease(&lease_id, LeaseStatus::Cancelled);
        }
        self.tasks[task_index].active_lease = None;
        self.tasks[task_index].available_at = now;
        self.tasks[task_index].run = next.clone();
        Ok(next)
    }

    fn resume(
        &mut self,
        task_id: TaskId,
        reason: String,
        now: Timestamp,
    ) -> Result<TaskRun, SchedulerError> {
        let task_index = self.task_index(task_id)?;
        let next = transition_task_state(
            &self.tasks[task_index].run,
            TaskTransition {
                from: TaskState::Paused,
                to: TaskState::Queued,
                reason,
                at: now,
            },
        )?;
        self.tasks[task_index].active_lease = None;
        self.tasks[task_index].available_at = now;
        self.tasks[task_index].run = next.clone();
        Ok(next)
    }

    fn release_expired_leases(&mut self, now: Timestamp) -> Vec<TaskLease> {
        let mut expired = Vec::new();
        for lease_index in 0..self.leases.len() {
            if self.leases[lease_index].status != LeaseStatus::Active
                || self.leases[lease_index].expires_at > now
            {
                continue;
            }

            self.leases[lease_index].status = LeaseStatus::Expired;
            let lease = self.leases[lease_index].clone();
            if let Ok(task_index) = self.task_index(lease.task_id)
                && self.tasks[task_index].active_lease.as_ref() == Some(&lease.id)
                && !self.tasks[task_index].run.state.is_terminal()
            {
                self.tasks[task_index].active_lease = None;
                self.tasks[task_index].available_at = now;
                let mut run = self.tasks[task_index].run.clone();
                run.state = TaskState::Queued;
                run.updated_at = now;
                self.tasks[task_index].run = run;
            }
            expired.push(lease);
        }
        expired
    }
}

#[derive(Clone, Debug)]
struct QueueEntry {
    run: TaskRun,
    available_at: Timestamp,
    active_lease: Option<TaskLeaseId>,
}

fn priority_rank(priority: TaskPriority) -> u8 {
    match priority {
        TaskPriority::High => 0,
        TaskPriority::Normal => 1,
        TaskPriority::Low => 2,
    }
}

fn add_millis(timestamp: Timestamp, millis: u64) -> Result<Timestamp, SchedulerError> {
    let millis = i64::try_from(millis)
        .map_err(|_| SchedulerError::InvalidLease("duration is too large".to_string()))?;
    Ok(Timestamp::from_utc(
        timestamp.as_offset_date_time() + Duration::milliseconds(millis),
    ))
}
