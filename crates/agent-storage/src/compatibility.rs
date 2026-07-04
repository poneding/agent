use agent_types::{ConversationId, RuntimeEvent, RuntimeEventPayload, TurnId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{CheckpointRecord, CheckpointStore, EventStore, StorageError};

pub const SUPPORTED_REPLAY_FIXTURE_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCompatibilityFixture {
    pub fixture_schema_version: u16,
    pub fixture_name: String,
    pub description: Option<String>,
    pub events: Vec<RuntimeEvent>,
    pub checkpoints: Vec<CheckpointRecord>,
    pub expected: ReplayCompatibilityExpected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayCompatibilityExpected {
    pub conversation_id: ConversationId,
    pub turn_id: TurnId,
    pub event_count: usize,
    pub checkpoint_count: usize,
    pub final_response: String,
    pub final_summary: String,
    pub audit_event_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayCompatibilityReport {
    pub fixture_name: String,
    pub event_count: usize,
    pub checkpoint_count: usize,
    pub compatible: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ReplayCompatibilityError {
    #[error("replay fixture is missing required field: {0}")]
    MissingField(String),
    #[error(
        "unsupported future replay fixture version {fixture_version}; supported version is {supported_version}"
    )]
    UnsupportedFutureVersion {
        fixture_version: u16,
        supported_version: u16,
    },
    #[error(
        "unsupported replay fixture version {fixture_version}; supported version is {supported_version}"
    )]
    UnsupportedVersion {
        fixture_version: u16,
        supported_version: u16,
    },
    #[error("replay fixture json is invalid: {0}")]
    Json(String),
    #[error("storage failed while loading replay fixture: {0}")]
    Storage(String),
    #[error("replay fixture is incompatible: {0}")]
    Incompatible(String),
}

#[derive(Debug, Deserialize)]
struct ReplayFixtureJson {
    fixture_schema_version: Option<u16>,
    fixture_name: Option<String>,
    description: Option<String>,
    events: Option<Vec<RuntimeEvent>>,
    checkpoints: Option<Vec<CheckpointRecord>>,
    expected: Option<serde_json::Value>,
}

pub fn load_replay_fixture_json(
    input: &str,
) -> Result<ReplayCompatibilityFixture, ReplayCompatibilityError> {
    let raw: ReplayFixtureJson = serde_json::from_str(input)
        .map_err(|error| ReplayCompatibilityError::Json(error.to_string()))?;
    let fixture_schema_version = required(raw.fixture_schema_version, "fixture_schema_version")?;

    if fixture_schema_version > SUPPORTED_REPLAY_FIXTURE_VERSION {
        return Err(ReplayCompatibilityError::UnsupportedFutureVersion {
            fixture_version: fixture_schema_version,
            supported_version: SUPPORTED_REPLAY_FIXTURE_VERSION,
        });
    }
    if fixture_schema_version != SUPPORTED_REPLAY_FIXTURE_VERSION {
        return Err(ReplayCompatibilityError::UnsupportedVersion {
            fixture_version: fixture_schema_version,
            supported_version: SUPPORTED_REPLAY_FIXTURE_VERSION,
        });
    }

    let expected = serde_json::from_value(required(raw.expected, "expected")?)
        .map_err(|error| ReplayCompatibilityError::Json(error.to_string()))?;
    let fixture = ReplayCompatibilityFixture {
        fixture_schema_version,
        fixture_name: required(raw.fixture_name, "fixture_name")?,
        description: raw.description,
        events: required(raw.events, "events")?,
        checkpoints: required(raw.checkpoints, "checkpoints")?,
        expected,
    };
    fixture.validate()?;
    Ok(fixture)
}

pub fn check_replay_compatibility(
    input: &str,
) -> Result<ReplayCompatibilityReport, ReplayCompatibilityError> {
    let fixture = load_replay_fixture_json(input)?;
    fixture.validate_expected_outcome()?;
    Ok(fixture.report(true))
}

impl ReplayCompatibilityFixture {
    pub fn load_into_store<S>(
        &self,
        store: &mut S,
    ) -> Result<ReplayCompatibilityReport, ReplayCompatibilityError>
    where
        S: EventStore + CheckpointStore,
    {
        for event in &self.events {
            store.append(event.clone()).map_err(replay_storage_error)?;
        }
        for checkpoint in &self.checkpoints {
            store
                .put_checkpoint(checkpoint.clone())
                .map_err(replay_storage_error)?;
        }
        Ok(self.report(true))
    }

    fn validate(&self) -> Result<(), ReplayCompatibilityError> {
        if self.events.len() != self.expected.event_count {
            return Err(ReplayCompatibilityError::Incompatible(format!(
                "expected {} events but fixture contains {}",
                self.expected.event_count,
                self.events.len()
            )));
        }
        if self.checkpoints.len() != self.expected.checkpoint_count {
            return Err(ReplayCompatibilityError::Incompatible(format!(
                "expected {} checkpoints but fixture contains {}",
                self.expected.checkpoint_count,
                self.checkpoints.len()
            )));
        }
        Ok(())
    }

    fn validate_expected_outcome(&self) -> Result<(), ReplayCompatibilityError> {
        let final_response = self
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                RuntimeEventPayload::FinalResponseProduced(response) => {
                    Some(response.message.as_str())
                }
                _ => None,
            })
            .next_back();
        let final_summary = self
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                RuntimeEventPayload::TurnCompleted(completed) => Some(completed.summary.as_str()),
                _ => None,
            })
            .next_back();
        let audit_event_count = self
            .events
            .iter()
            .filter(|event| is_audit_payload(&event.payload))
            .count();

        if final_response != Some(self.expected.final_response.as_str()) {
            return Err(ReplayCompatibilityError::Incompatible(
                "final response did not match fixture expectation".to_string(),
            ));
        }
        if final_summary != Some(self.expected.final_summary.as_str()) {
            return Err(ReplayCompatibilityError::Incompatible(
                "final summary did not match fixture expectation".to_string(),
            ));
        }
        if audit_event_count != self.expected.audit_event_count {
            return Err(ReplayCompatibilityError::Incompatible(format!(
                "expected {} audit events but fixture contains {}",
                self.expected.audit_event_count, audit_event_count
            )));
        }
        Ok(())
    }

    fn report(&self, compatible: bool) -> ReplayCompatibilityReport {
        ReplayCompatibilityReport {
            fixture_name: self.fixture_name.clone(),
            event_count: self.events.len(),
            checkpoint_count: self.checkpoints.len(),
            compatible,
        }
    }
}

fn required<T>(value: Option<T>, field: &str) -> Result<T, ReplayCompatibilityError> {
    value.ok_or_else(|| ReplayCompatibilityError::MissingField(field.to_string()))
}

fn is_audit_payload(payload: &RuntimeEventPayload) -> bool {
    matches!(
        payload,
        RuntimeEventPayload::ToolCallProposed(_)
            | RuntimeEventPayload::PermissionDecisionRecorded(_)
            | RuntimeEventPayload::ToolExecutionStarted(_)
            | RuntimeEventPayload::ToolExecutionCompleted(_)
            | RuntimeEventPayload::ToolExecutionFailed(_)
            | RuntimeEventPayload::ToolObservationRecorded(_)
    )
}

fn replay_storage_error(error: StorageError) -> ReplayCompatibilityError {
    ReplayCompatibilityError::Storage(error.to_string())
}
