use std::fmt;
use std::num::NonZeroU64;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AgentTypesError;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }
    };
}

uuid_id!(WorkspaceId);
uuid_id!(SessionId);
uuid_id!(ConversationId);
uuid_id!(TurnId);
uuid_id!(TaskId);
uuid_id!(AgentId);
uuid_id!(TeamId);
uuid_id!(EventId);
uuid_id!(CorrelationId);
uuid_id!(CheckpointId);
uuid_id!(ApprovalRequestId);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSeq(NonZeroU64);

impl EventSeq {
    pub fn new(value: u64) -> Result<Self, AgentTypesError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(AgentTypesError::InvalidEventSequence(value))
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for EventSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_through_string() {
        let original = WorkspaceId::from_uuid(Uuid::nil());
        let parsed: WorkspaceId = original.to_string().parse().expect("valid uuid");
        assert_eq!(parsed, original);
    }

    #[test]
    fn id_serializes_as_uuid_string() {
        let id = EventId::from_uuid(Uuid::nil());
        let json = serde_json::to_string(&id).expect("serialize id");
        assert_eq!(json, "\"00000000-0000-0000-0000-000000000000\"");
    }

    #[test]
    fn event_sequence_rejects_zero() {
        let error = EventSeq::new(0).expect_err("zero sequence is invalid");
        assert_eq!(error.to_string(), "event sequence must be non-zero: 0");
    }

    #[test]
    fn event_sequence_accepts_positive_values() {
        let seq = EventSeq::new(7).expect("positive sequence");
        assert_eq!(seq.get(), 7);
    }
}
