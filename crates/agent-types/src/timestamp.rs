use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(#[serde(with = "time::serde::rfc3339")] OffsetDateTime);

impl Timestamp {
    #[must_use]
    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    #[must_use]
    pub const fn from_utc(value: OffsetDateTime) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_offset_date_time(self) -> OffsetDateTime {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn timestamp_serializes_as_rfc3339() {
        let timestamp = Timestamp::from_utc(datetime!(2026-06-25 00:00:00 UTC));
        let json = serde_json::to_string(&timestamp).expect("serialize timestamp");
        assert_eq!(json, "\"2026-06-25T00:00:00Z\"");
    }
}
